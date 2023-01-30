use common::geo::{Vector2, Rect2};
use std::alloc::{Layout, alloc_zeroed};

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Pixel {
    b:     u8,
    g:     u8,
    r:     u8,
    __pad: u8,
}

impl Pixel {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            __pad: 0
        }
    }
}

impl From<(u8, u8, u8)> for Pixel {
    fn from(value: (u8, u8, u8)) -> Self {
        Pixel::new(value.0, value.1, value.2)
    }
}

pub struct Bitmap {
    pub size: Rect2,
    layout:   Layout,
    memory:   *mut Pixel,
}

pub enum RenderError {
    /// attempted to draw out of bounds
    DrawOOB,
    MemoryError
}

impl Bitmap {
    pub fn new(size: Rect2) -> Self {
        let layout = Layout::array::<Pixel>(size.area() as usize)
            .expect("[tkym-graphics] bitmap memory allocation failed");
        let memory = unsafe {
            alloc_zeroed(layout)
        }.cast();
        Self {
            size,
            layout,
            memory
        }
    }

    pub fn pixel_at_point<T: Into<Vector2>>(&self, point: T) -> Option<&Pixel> {
        let Rect2 { width, height } = self.size;
        let Vector2 { x, y } = point.into();
        let index = width * y + x;
        debug_assert!(index < width * height);
        unsafe {
            self.memory.offset(index as isize)
                .as_ref()
        }
    }

    pub fn pixel_at_point_mut<T: Into<Vector2>>(&mut self, point: T) -> Result<&mut Pixel, RenderError> {
        let Rect2 { width, height } = self.size;
        let Vector2 { x, y } = point.into();
        let index = width * y + x;
        if index > width * height {
            return Err(RenderError::DrawOOB)
        }
        unsafe {
            match self.memory.offset(index as isize)
            .as_mut() {
                None => return Err(RenderError::MemoryError),
                Some(pixel) => return Ok(pixel)
            }
        }
    }

    pub fn draw_point<
        Pt: Into<Vector2>, 
        Px: Into<Pixel>
    > ( 
        &mut self,
        point: Pt, 
        pixel: Px
    ) {
        if let Ok(old_pixel) = self.pixel_at_point_mut(point.into()) {
            *old_pixel = pixel.into();
        } 
    }

    pub fn draw_rect<
        Rct: Into<Rect2>,
        Pt:  Into<Vector2>,
        Px:  Into<Pixel> + Copy,
    > (
        &mut self,
        offset: Pt,
        rect:   Rct,
        pixel:  Px
    ) {
        let offset = offset.into();
        let rect = rect.into();

        for mut x in 0..=rect.width {
            x += offset.x;
            for mut y in 0..=rect.height {
                y += offset.y;
                self.draw_point(
                    (x, y),
                    pixel,
                );
            }
        }
    }
}

impl Into<*const u8> for &Bitmap {
    fn into(self) -> *const u8 {
        self.memory.cast()
    }
}

impl Drop for Bitmap {
    fn drop(&mut self) {
        use std::alloc::dealloc;
        unsafe {
            dealloc(self.memory.cast(), self.layout);
        }
    }
}

