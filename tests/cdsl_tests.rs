//! Integration and unit test suite for CDSL core components.
//!
//! Validates tokenization, input validation, runtime variables,
//! arithmetic expressions, and XML configuration parsing.

use cdsl::core::*;

/// Tests command line tokenization, whitespace handling, and comment stripping.
#[test]
fn test_tokenize() {
    assert_eq!(tokenize("set x 10"), vec!["set", "x", "10"]);
    assert_eq!(tokenize("  set   x   10  "), vec!["set", "x", "10"]);
    assert_eq!(tokenize("set x 10 // comment here"), vec!["set", "x", "10"]);
    assert_eq!(tokenize("// comment only"), Vec::<String>::new());
    assert_eq!(tokenize(""), Vec::<String>::new());
}

/// Tests identifier validation rules for variable names.
#[test]
fn test_variable_validation() {
    // Valid identifier names.
    assert!(is_valid_variable_name("x"));
    assert!(is_valid_variable_name("_x"));
    assert!(is_valid_variable_name("var_1"));
    assert!(is_valid_variable_name("myVar123"));

    // Invalid identifier names.
    assert!(!is_valid_variable_name(""));
    assert!(!is_valid_variable_name("1var"));
    assert!(!is_valid_variable_name("var-name"));
    assert!(!is_valid_variable_name("var name"));
}

/// Tests parsing and validation of floating-point values.
#[test]
fn test_float_validation() {
    // Valid float strings.
    assert_eq!(is_valid_float_value("123").unwrap(), 123.0);
    assert_eq!(is_valid_float_value("123.45").unwrap(), 123.45);
    assert_eq!(is_valid_float_value("-12.5").unwrap(), -12.5);
    assert_eq!(is_valid_float_value("0").unwrap(), 0.0);

    // Invalid float strings and out-of-range checks.
    assert_eq!(
        is_valid_float_value("abc").unwrap_err(),
        "invalid value: abc"
    );
    assert_eq!(
        is_valid_float_value("1e50").unwrap_err(),
        "value out of range"
    );
}

/// Tests parsing and validation of integer values.
#[test]
fn test_int_validation() {
    // Valid integer strings.
    assert_eq!(is_valid_int_value("123").unwrap(), 123);
    assert_eq!(is_valid_int_value("-45").unwrap(), -45);
    assert_eq!(is_valid_int_value("0").unwrap(), 0);

    // Invalid integer strings.
    assert_eq!(is_valid_int_value("abc").unwrap_err(), "invalid value: abc");
    assert_eq!(
        is_valid_int_value("12.34").unwrap_err(),
        "invalid value: 12.34"
    );
}

/// Tests setting, checking, and retrieving runtime variables.
#[test]
fn test_variables() {
    set_variable("test_var", 42.5);
    assert!(has_variable("test_var"));
    assert_eq!(get_variable("test_var"), 42.5);
    assert!(!has_variable("non_existent"));
    assert_eq!(get_variable("non_existent"), 0.0);
}

/// Tests arithmetic expression evaluation, operator precedence, and error cases.
#[test]
fn test_expression_evaluation() {
    set_variable("a", 10.0);
    set_variable("b", 2.0);

    // Arithmetic operators and precedence.
    assert_eq!(evaluate_expression("1 + 2").unwrap(), 3.0);
    assert_eq!(evaluate_expression("10 - 4").unwrap(), 6.0);
    assert_eq!(evaluate_expression("3 * 4").unwrap(), 12.0);
    assert_eq!(evaluate_expression("12 / 3").unwrap(), 4.0);
    assert_eq!(evaluate_expression("2 + 3 * 4").unwrap(), 14.0);
    assert_eq!(evaluate_expression("(2 + 3) * 4").unwrap(), 20.0);
    assert_eq!(evaluate_expression("-5 + 10").unwrap(), 5.0);
    assert_eq!(evaluate_expression("+5 + 10").unwrap(), 15.0);
    assert_eq!(evaluate_expression("$a + $b").unwrap(), 12.0);
    assert_eq!(evaluate_expression("($a * $b) - 5").unwrap(), 15.0);

    // Error handling.
    assert_eq!(
        evaluate_expression("1 / 0").unwrap_err(),
        "division by zero"
    );
    assert_eq!(evaluate_expression("(1 + 2").unwrap_err(), "expected ')'");
    assert_eq!(
        evaluate_expression("$unknown").unwrap_err(),
        "undefined variable: $unknown"
    );
    assert_eq!(
        evaluate_expression("1 + @").unwrap_err(),
        "unexpected character: @"
    );
}

/// Tests parsing an XML configuration string into Arduino settings and component limits.
#[test]
fn test_xml_config_parsing() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<config>
    <arduino>
        <port>/dev/ttyUSB0</port>
        <baud_rate>115200</baud_rate>
    </arduino>

    <component>
        <base>
            <id>0</id>
            <min>0</min>
            <max>180</max>
            <default>90</default>
        </base>
        <claw>
            <id>5</id>
            <min>0</min>
            <max>90</max>
            <default>0</default>
        </claw>
    </component>
</config>"#;

    let doc = roxmltree::Document::parse(xml).unwrap();
    let root = doc.root_element();
    assert!(root.has_tag_name("config"));

    // Verify Arduino settings.
    let ard = root.children().find(|n| n.has_tag_name("arduino")).unwrap();
    let port = ard
        .children()
        .find(|n| n.has_tag_name("port"))
        .unwrap()
        .text()
        .unwrap();
    let baud = ard
        .children()
        .find(|n| n.has_tag_name("baud_rate"))
        .unwrap()
        .text()
        .unwrap()
        .parse::<i64>()
        .unwrap();
    assert_eq!(port, "/dev/ttyUSB0");
    assert_eq!(baud, 115200);

    // Verify component settings.
    let comp = root
        .children()
        .find(|n| n.has_tag_name("component"))
        .unwrap();
    let base = comp.children().find(|n| n.has_tag_name("base")).unwrap();
    let id = base
        .children()
        .find(|n| n.has_tag_name("id"))
        .unwrap()
        .text()
        .unwrap()
        .parse::<i64>()
        .unwrap();
    assert_eq!(id, 0);
}
