use qrcodegen::{QrCode, QrCodeEcc};

use druid::widget::Widget;
use druid::{BoxConstraints, Geometry, Id, LayoutCtx, LayoutResult, PaintCtx, Ui};

use kurbo::Rect;
use piet::{FillRule, ImageFormat, InterpolationMode, RenderContext};

const BOX_HEIGHT: f64 = 256.;

pub struct Qr {
    value: String,
}

impl Qr {
    pub fn new(value: String) -> Qr {
        Qr { value }
    }
    pub fn ui(self, ctx: &mut Ui) -> Id {
        ctx.add(self, &[])
    }
}

impl Widget for Qr {
    fn paint(&mut self, paint_ctx: &mut PaintCtx, geom: &Geometry) {
        let white = 0xff_ff_ff_ff;

        let (x, y) = geom.pos;
        let (x, y) = (x as f64, y as f64);

        // Draw a white background so we get some white padding
        let brush = paint_ctx.render_ctx.solid_brush(white).unwrap();

        let rect = Rect::new(x, y, x + BOX_HEIGHT, y + BOX_HEIGHT);

        paint_ctx.render_ctx.fill(rect, &brush, FillRule::NonZero);

        // Generate the QR code from the given text, at medium error correction
        let qr = QrCode::encode_text(&self.value, QrCodeEcc::Medium).unwrap();

        let size = qr.size() as usize;

        let mut image_data = vec![255; size * size * 4];
        for y in 0..size {
            for x in 0..size {
                let ix = (y * size + x) * 4;
                if qr.get_module(x as i32, y as i32) {
                    image_data[ix + 0] = 0;
                    image_data[ix + 1] = 0;
                    image_data[ix + 2] = 0;
                }

                image_data[ix + 3] = 255;
            }
        }

        let image = paint_ctx
            .render_ctx
            .make_image(size, size, &image_data, ImageFormat::RgbaSeparate)
            .unwrap();

        paint_ctx.render_ctx.draw_image(
            &image,
            (
                (x + 10., y + 10.),
                (x + BOX_HEIGHT - 10., y + BOX_HEIGHT - 10.),
            ),
            InterpolationMode::NearestNeighbor,
        );
    }

    fn layout(
        &mut self,
        bc: &BoxConstraints,
        _children: &[Id],
        _size: Option<(f32, f32)>,
        _ctx: &mut LayoutCtx,
    ) -> LayoutResult {
        LayoutResult::Size(bc.constrain((BOX_HEIGHT as f32, BOX_HEIGHT as f32)))
    }
}
