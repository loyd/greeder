extern crate handlebars;

use self::handlebars::{Handlebars,
    RenderError,
    RenderContext,
    Helper,
    JsonRender,
    Decorator
};

fn format_date(h: &Helper, hb: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    // let param = try!(h.param(0).ok_or(RenderError::new("Param 0 is required for format helper.")));
    let param = try!(h.param(0).ok_or(RenderError::new("Param 0 is required for format helper.")));
    let rendered = format!("{} pts", param.value().render());
    rc.writer.write(rendered.into_bytes().as_ref());
    Ok(())
}

pub fn setup() {
    let mut handlebars = Handlebars::new();
    handlebars.register_helper("ydz", Box::new(format_date));
}