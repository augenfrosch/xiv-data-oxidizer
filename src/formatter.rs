use ironworks::sestring::{
    Error as SeStringError, SeString,
    format::{Color, ColorUsage, Input, Style, Write, format},
};

pub fn format_string(string: &SeString, input: &Input) -> String {
    let mut writer = MarkdownWriter::default();
    let _ = format(string.as_ref(), &input, &mut writer);

    return writer.buffer;
}

#[derive(Debug, Default)]
struct MarkdownWriter {
    buffer: String,
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
