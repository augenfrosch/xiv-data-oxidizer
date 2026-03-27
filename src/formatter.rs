use std::fmt::{self, Display};

use ironworks::sestring::{
    Error as SeStringError, Expression, SeString,
    format::{Color, ColorUsage, Input, Style, Write, format},
};

pub fn format_string<Writer>(string: &SeString, input: &Input, mut writer: Writer) -> String
where
    Writer: Write + Into<String>,
{
    let _ = format(string.as_ref(), &input, &mut writer);

    return writer.into();
}

#[derive(Debug, Default)]
pub struct MarkdownWriter {
    buffer: String,
}

impl From<MarkdownWriter> for String {
    fn from(writer: MarkdownWriter) -> Self {
        writer.buffer
    }
}

impl Write for MarkdownWriter {
    fn write_str(&mut self, str: &str) -> Result<(), SeStringError> {
        self.buffer.push_str(&str.replace('*', r"\*"));

        Ok(())
    }

    // Format styled text with markdown
    fn set_style(&mut self, style: Style, enabled: bool) -> Result<(), SeStringError> {
        let markdown = match style {
            Style::Bold => "**",
            Style::Italic => "*",
            _ => return Ok(()),
        };
        let idx = if enabled {
            self.buffer.len()
        } else {
            self.buffer.trim_end().len()
        };

        self.buffer.insert_str(idx, markdown);

        Ok(())
    }

    // Just replace all of the color shenanigans with bold markdown
    fn push_color(&mut self, usage: ColorUsage, _color: Color) -> Result<(), SeStringError> {
        if usage == ColorUsage::Foreground {
            self.buffer.push_str("**");
        }

        Ok(())
    }

    fn pop_color(&mut self, usage: ColorUsage) -> Result<(), SeStringError> {
        if usage == ColorUsage::Foreground {
            self.buffer.insert_str(self.buffer.trim_end().len(), "**");
        }

        Ok(())
    }
}

// Adapted from boilmaster's implementation (mostly just simplified)
// See: https://github.com/ackwell/boilmaster/blob/main/crates/bm_http/src/api1/string.rs
#[derive(Debug, Default)]
pub struct HtmlWriter {
    buffer: String,
}

impl From<HtmlWriter> for String {
    fn from(writer: HtmlWriter) -> Self {
        writer.buffer
    }
}

impl Write for HtmlWriter {
    fn write_str(&mut self, str: &str) -> Result<(), SeStringError> {
        // TODO check what else has to be replaces and/or escaped to be valid HTML text; boilmaster escapes a few chars
        self.buffer.push_str(&str.replace('\n', "<br>"));

        Ok(())
    }

    fn set_style(&mut self, style: Style, enabled: bool) -> Result<(), SeStringError> {
        let tag = match (style, enabled) {
            (Style::Bold, true) => "<b>",
            (Style::Bold, false) => "</b>",
            (Style::Italic, true) => "<em>",
            (Style::Italic, false) => "</em>",
            _ => return Ok(()),
        };
        self.buffer.push_str(tag);

        Ok(())
    }

    fn push_color(&mut self, usage: ColorUsage, color: Color) -> Result<(), SeStringError> {
        if usage != ColorUsage::Foreground {
            return Ok(());
        }

        let Color { r, g, b, a } = color;
        self.buffer.push_str(&format!(
            r#"<span style="color:#{r:02x}{g:02x}{b:02x}{a:02x}">"#
        ));

        Ok(())
    }

    fn pop_color(&mut self, usage: ColorUsage) -> Result<(), SeStringError> {
        if usage != ColorUsage::Foreground {
            return Ok(());
        }

        self.buffer.push_str("</span>");

        Ok(())
    }
}

// Directly based on what Asriel does for her ironworks fork (without modifying ironworks directly)
// See: https://github.com/WorkingRobot/ironworks/blob/main/ironworks/src/sestring/macro_string.rs
// TODO look at the differences to the original implementation and check if
#[derive(Debug)]
pub struct MacroString<'a> {
    sestring: SeString<'a>,
}

impl fmt::Display for MacroString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Self::fmt_sestring(&self.sestring, f)
    }
}

impl<'a> MacroString<'a> {
    pub fn new(sestring: SeString<'a>) -> Self {
        Self { sestring }
    }

    fn fmt_sestring(sestring: &SeString, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for payload in sestring.payloads() {
            let payload = payload.expect("Invalid macro");
            match payload {
                ironworks::sestring::Payload::Text(text_payload) => {
                    formatter.write_str(text_payload.as_utf8().expect("Invalid text"))?;
                }
                ironworks::sestring::Payload::Macro(macro_payload) => {
                    let kind = macro_payload.kind();
                    write!(formatter, "<{kind:?}")?;

                    let expressions = macro_payload.expressions();
                    let has_expressions = expressions.peekable().peek().is_some();
                    if has_expressions {
                        formatter.write_str("(")?;
                        for (i, expression) in macro_payload.expressions().enumerate() {
                            if i != 0 {
                                formatter.write_str(",")?;
                            }
                            let expression = expression.expect("Invalid Expression");
                            Self::fmt_expression(&expression, formatter)?;
                        }
                        formatter.write_str(")")?;
                    }

                    formatter.write_str(">")?;
                }
            }
        }

        Ok(())
    }

    fn fmt_expression(
        expression: &Expression<'_>,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match expression {
            Expression::U32(value) => {
                value.fmt(formatter)?;
            }
            Expression::SeString(sestring) => {
                Self::fmt_sestring(sestring, formatter)?;
            }
            Expression::Millisecond
            | Expression::Second
            | Expression::Minute
            | Expression::Hour
            | Expression::Day
            | Expression::Weekday
            | Expression::Month
            | Expression::Year
            | Expression::StackColor => {
                formatter.write_str(Self::characteristic_str(expression))?;
            }
            Expression::LocalNumber(e)
            | Expression::GlobalNumber(e)
            | Expression::LocalString(e)
            | Expression::GlobalString(e) => {
                formatter.write_str(Self::characteristic_str(expression))?;
                Self::fmt_expression(e, formatter)?;
            }
            Expression::Ge(lhs, rhs)
            | Expression::Gt(lhs, rhs)
            | Expression::Le(lhs, rhs)
            | Expression::Lt(lhs, rhs)
            | Expression::Eq(lhs, rhs)
            | Expression::Ne(lhs, rhs) => {
                formatter.write_str("[")?;
                Self::fmt_expression(lhs, formatter)?;
                formatter.write_str(Self::characteristic_str(expression))?;
                Self::fmt_expression(rhs, formatter)?;
                formatter.write_str("]")?;
            }
            Expression::Unknown(value) => {
                write!(formatter, "unknown({value})")?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn characteristic_str(expression: &Expression<'_>) -> &'static str {
        match expression {
            Expression::U32(_) => "u32",
            Expression::SeString(_) => "sestring",
            Expression::Millisecond => "t_msec",
            Expression::Second => "t_sec",
            Expression::Minute => "t_min",
            Expression::Hour => "t_hour",
            Expression::Day => "t_day",
            Expression::Weekday => "t_wday",
            Expression::Month => "t_month",
            Expression::Year => "t_year",
            Expression::StackColor => "stackcolor",
            Expression::LocalNumber(_) => "lnum",
            Expression::GlobalNumber(_) => "gnun",
            Expression::LocalString(_) => "lstr",
            Expression::GlobalString(_) => "gstr",
            Expression::Ge(_, _) => ">=",
            Expression::Gt(_, _) => ">",
            Expression::Le(_, _) => "<=",
            Expression::Lt(_, _) => "<",
            Expression::Eq(_, _) => "==",
            Expression::Ne(_, _) => "!=",
            Expression::Unknown(_) => "unknown",
            _ => unreachable!(),
        }
    }
}
