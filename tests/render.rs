mod support;

use insta::assert_snapshot;
use support::{MULTI, SINGLE, chart};

const NEGATIVE: &str = "a,-5,3\nb,10,2\nc,0,1\n";
const LARGE: &str = "x,12550\ny,19561100\nz,999\n";

#[test]
fn single_series() {
    assert_snapshot!(chart(SINGLE, &["--width", "20"]));
}

#[test]
fn single_series_colored_with_title() {
    assert_snapshot!(chart(SINGLE, &["--width", "20", "--color", "red", "--title", "Births"]));
}

#[test]
fn multi_series() {
    assert_snapshot!(chart(MULTI, &["--width", "20", "--color", "blue", "red"]));
}

#[test]
fn stacked() {
    assert_snapshot!(chart(MULTI, &["--width", "20", "--stacked"]));
}

#[test]
fn stacked_spaced_with_suffix() {
    assert_snapshot!(chart(MULTI, &["--width", "20", "--stacked", "--space-between", "--suffix", "%"]));
}

#[test]
fn negative_values() {
    assert_snapshot!(chart(NEGATIVE, &["--width", "20"]));
}

#[test]
fn label_before() {
    assert_snapshot!(chart(SINGLE, &["--width", "20", "--label-before"]));
}

#[test]
fn hidden_labels_and_values() {
    assert_snapshot!(chart(MULTI, &["--width", "20", "--no-labels", "--no-values", "--suffix", "u"]));
}

#[test]
fn custom_tick() {
    assert_snapshot!(chart(SINGLE, &["--width", "20", "--custom-tick", "#"]));
}

#[test]
fn scaled_values_with_custom_format() {
    assert_snapshot!(chart(LARGE, &["--width", "20", "--format", "%.1f"]));
}

#[test]
fn small_values_are_not_scaled_up() {
    assert_snapshot!(chart("a,0.5\nb,2\n", &["--width", "20"]));
}

#[test]
fn custom_delimiter_and_quoted_label() {
    assert_snapshot!(chart("\"x; y\";1;2\nz;3;4\n", &["--width", "20", "--delim", ";"]));
}

#[test]
fn legend_wraps_at_width() {
    assert_snapshot!(chart("@,Alpha,Bravo,Charlie\nr,1,2,3\n", &["--width", "12"]));
}
