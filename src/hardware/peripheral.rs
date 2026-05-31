use crate::hardware::{processor::Processor, word::Word};
use raylib::core::texture::Image;

pub trait Peripheral {
    /// Called after each processor tick. If the peripheral emits an interrupt, return Some(message).
    fn tick(&mut self, memory:&mut [Word]) -> Option<Word>;

    /// Called after the processor has performed a hardware interrupt targeting this peripheral. Must return the number
    /// of additional cycles that the processor gets stalled for, even if this is 0 additional cycles.
    fn interrupt(&mut self, memory:&mut [Word], registers:&mut Processor) -> u16;

    /// Should return the width and height of the display image for this peripheral. Must remain consistent between
    /// calls!
    fn render_size(&self) -> (u32,u32);

    /// Render the current state to the given image, then return said image. The image will be of the size given by render_size().
    fn render_image(&self, destination:Image) -> Image;
}