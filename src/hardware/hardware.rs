//! Serial communication driver for sending commands to Arduino hardware.

use std::ffi::CString;
use std::io::{self, Write};
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::core::{arduino_baud_rate, arduino_port, component_id, Config};

// Standard baud rate definitions.
const BAUD_9600: i32 = 9600;
const BAUD_19200: i32 = 19200;
const BAUD_38400: i32 = 38400;
const BAUD_57600: i32 = 57600;
const BAUD_115200: i32 = 115200;

// Maximum duration to wait for an Arduino response before timing out.
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);

// RAII wrapper to ensure a POSIX file descriptor is automatically closed on drop.
struct FdGuard(libc::c_int);

impl Drop for FdGuard {
    fn drop(&mut self) {
        if self.0 >= 0 {
            unsafe {
                libc::close(self.0);
            }
        }
    }
}

// Maps an integer baud rate to its corresponding termios speed constant.
fn get_baud_rate(baud_rate: i32) -> Result<libc::speed_t, String> {
    match baud_rate {
        BAUD_9600 => Ok(libc::B9600),
        BAUD_19200 => Ok(libc::B19200),
        BAUD_38400 => Ok(libc::B38400),
        BAUD_57600 => Ok(libc::B57600),
        BAUD_115200 => Ok(libc::B115200),
        _ => Err(format!("unsupported baud rate: {}", baud_rate)),
    }
}

// Configures the serial port for raw 8-N-1 communication (8 data bits, no parity, 1 stop bit).
fn configure_serial(fd: libc::c_int, speed: libc::speed_t) -> Result<(), String> {
    unsafe {
        let mut termios: libc::termios = std::mem::zeroed();
        if libc::tcgetattr(fd, &mut termios) != 0 {
            return Err(format!(
                "get serial configuration: {}",
                io::Error::last_os_error()
            ));
        }

        // Configure raw input: disable break handling, parity checking, and software flow control.
        termios.c_iflag &= !(libc::IGNBRK
            | libc::BRKINT
            | libc::PARMRK
            | libc::ISTRIP
            | libc::INLCR
            | libc::IGNCR
            | libc::ICRNL
            | libc::IXON
            | libc::IXOFF
            | libc::IXANY);

        // Disable post-processing of output.
        termios.c_oflag &= !libc::OPOST;

        // Disable echo, canonical line buffering, and signal generation.
        termios.c_lflag &= !(libc::ECHO | libc::ECHONL | libc::ICANON | libc::ISIG | libc::IEXTEN);

        // Configure 8-N-1 framing (8 data bits, no parity, 1 stop bit).
        termios.c_cflag &= !(libc::CSIZE | libc::PARENB | libc::CSTOPB | libc::CBAUD);
        termios.c_cflag |= libc::CS8 | libc::CREAD | libc::CLOCAL;

        // Set input and output baud rate.
        libc::cfsetspeed(&mut termios, speed);

        // Set minimum characters for non-canonical read; polling handles timeouts.
        termios.c_cc[libc::VMIN] = 1;
        termios.c_cc[libc::VTIME] = 0;

        if libc::tcsetattr(fd, libc::TCSANOW, &termios) != 0 {
            return Err(format!(
                "set serial configuration: {}",
                io::Error::last_os_error()
            ));
        }
    }

    Ok(())
}

// Writes all bytes to the file descriptor, retrying if interrupted by signals.
fn write_all(fd: libc::c_int, mut data: &[u8]) -> Result<(), String> {
    while !data.is_empty() {
        let n = unsafe {
            libc::write(
                fd,
                data.as_ptr() as *const libc::c_void,
                data.len() as libc::size_t,
            )
        };
        if n < 0 {
            let err = io::Error::last_os_error();
            if err.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(err.to_string());
        }
        if n == 0 {
            return Err("zero bytes written".to_string());
        }
        data = &data[n as usize..];
    }

    Ok(())
}

// Reads bytes from the serial port until a newline character is encountered or timeout occurs.
fn read_line(fd: libc::c_int) -> Result<String, String> {
    let deadline = Instant::now() + RESPONSE_TIMEOUT;
    let mut response = Vec::new();

    loop {
        let now = Instant::now();
        if now >= deadline {
            return Err("timed out waiting for Arduino response".to_string());
        }

        let remaining = deadline - now;
        let timeout_ms = remaining.as_millis().max(1) as libc::c_int;

        let mut fds = [libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        }];

        // Wait for incoming data on the serial port.
        let n = unsafe { libc::poll(fds.as_mut_ptr(), 1, timeout_ms) };
        if n < 0 {
            let err = io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            return Err(format!("poll serial port: {}", err));
        }
        if n == 0 {
            return Err("timed out waiting for Arduino response".to_string());
        }

        let revents = fds[0].revents;
        if (revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL)) != 0 {
            return Err(format!("serial port error: events {:#x}", revents));
        }

        if (revents & libc::POLLIN) == 0 {
            continue;
        }

        // Read one byte at a time.
        let mut b = [0u8; 1];
        let count = unsafe { libc::read(fd, b.as_mut_ptr() as *mut libc::c_void, 1) };

        if count < 0 {
            let err = io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            return Err(format!("read serial port: {}", err));
        }
        if count == 0 {
            continue;
        }

        // When newline is reached, return the accumulated line without trailing carriage returns.
        if b[0] == b'\n' {
            if response.last() == Some(&b'\r') {
                response.pop();
            }
            return Ok(String::from_utf8_lossy(&response).into_owned());
        }

        response.push(b[0]);
    }
}

/// Sends a component movement command to the Arduino and validates the response.
///
/// Format sent: `<component_id> <float_value>\n`
/// Expected responses: `EXECUTED: <component_id> <value>` or `REJECTED`.
pub fn send_command(config: &Config, component_label: &str, value: f32) -> Result<(), String> {
    // Validate that the value is a valid numeric range for servo / hardware control.
    if value.is_nan() || value.is_infinite() {
        return Err(format!("invalid value: {}", value));
    }

    if !(0.0..=180.0).contains(&value) {
        return Err(format!("value must be between 0 and 180: {}", value));
    }

    // Resolve component ID and serial port configuration.
    let comp_id = component_id(config, component_label)
        .map_err(|e| format!("get component ID for {:?}: {}", component_label, e))?;

    let configured_baud =
        arduino_baud_rate(config).map_err(|e| format!("get Arduino baud rate: {}", e))?;

    let baud_rate =
        get_baud_rate(configured_baud).map_err(|e| format!("configure baud rate: {}", e))?;

    let port = arduino_port(config).map_err(|e| format!("get Arduino port: {}", e))?;

    let c_port = CString::new(port.clone()).map_err(|e| format!("invalid port string: {}", e))?;

    // Open serial port device.
    let fd = unsafe { libc::open(c_port.as_ptr(), libc::O_RDWR | libc::O_NOCTTY) };
    if fd < 0 {
        return Err(format!(
            "open serial port {:?}: {}",
            port,
            io::Error::last_os_error()
        ));
    }
    let _guard = FdGuard(fd);

    configure_serial(fd, baud_rate)
        .map_err(|e| format!("configure serial port {:?}: {}", port, e))?;

    // Many Arduino boards auto-reset when a serial connection is established.
    // Wait 2 seconds for bootloader initialization.
    println!("Waiting for Arduino to initialize...");
    sleep(Duration::from_secs(2));

    // Clear any startup messages from the input buffer.
    unsafe {
        if libc::tcflush(fd, libc::TCIFLUSH) != 0 {
            return Err(format!(
                "flush serial input: {}",
                io::Error::last_os_error()
            ));
        }
    }

    // Transmit command formatted as: "<component_id> <value>\n"
    let command = format!("{} {:.6}\n", comp_id, value);

    print!("Sending: {}", command);
    io::stdout().flush().ok();

    write_all(fd, command.as_bytes()).map_err(|e| format!("send command to Arduino: {}", e))?;

    println!("Waiting for Arduino response...");
    io::stdout().flush().ok();

    // Read response line from Arduino.
    let response = read_line(fd).map_err(|e| format!("receive Arduino response: {}", e))?;

    println!("Received: {}", response);

    if response == "REJECTED" {
        return Err(format!(
            "arduino rejected component {} with value {:.6}",
            comp_id, value
        ));
    }

    let expected_prefix = format!("EXECUTED: {} ", comp_id);
    if !response.starts_with(&expected_prefix) {
        return Err(format!("unexpected Arduino response: {:?}", response));
    }

    println!("Command acknowledged successfully.");

    Ok(())
}
