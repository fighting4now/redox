//! # Area frame allocator
//! Some code was borrowed from [Phil Opp's Blog](http://os.phil-opp.com/allocating-frames.html)

use paging::PhysicalAddress;

use super::{Frame, FrameAllocator, MemoryArea, MemoryAreaIter};


pub struct AreaFrameAllocator {
    next_free_frame: Frame,
    current_area: Option<&'static MemoryArea>,
    areas: MemoryAreaIter,
    kernel_start: Frame,
    kernel_end: Frame
}

impl AreaFrameAllocator {
    pub fn new(kernel_start: usize, kernel_end: usize, memory_areas: MemoryAreaIter) -> AreaFrameAllocator {
        let mut allocator = AreaFrameAllocator {
            next_free_frame: Frame::containing_address(PhysicalAddress::new(0)),
            current_area: None,
            areas: memory_areas,
            kernel_start: Frame::containing_address(PhysicalAddress::new(kernel_start)),
            kernel_end: Frame::containing_address(PhysicalAddress::new(kernel_end))
        };
        allocator.choose_next_area();
        allocator
    }

    fn choose_next_area(&mut self) {
        self.current_area = self.areas.clone().filter(|area| {
            let address = area.base_addr + area.length - 1;
            Frame::containing_address(PhysicalAddress::new(address as usize)) >= self.next_free_frame
        }).min_by_key(|area| area.base_addr);

        if let Some(area) = self.current_area {
            let start_frame = Frame::containing_address(PhysicalAddress::new(area.base_addr as usize));
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}

impl FrameAllocator for AreaFrameAllocator {
    fn allocate_frames(&mut self, count: usize) -> Option<Frame> {
        if count == 0 {
            None
        } else if let Some(area) = self.current_area {
            // "Clone" the frame to return it if it's free. Frame doesn't
            // implement Clone, but we can construct an identical frame.
            let start_frame = Frame{ number: self.next_free_frame.number };
            let end_frame = Frame { number: self.next_free_frame.number + (count - 1) };

            // the last frame of the current area
            let current_area_last_frame = {
                let address = area.base_addr + area.length - 1;
                Frame::containing_address(PhysicalAddress::new(address as usize))
            };

            if end_frame > current_area_last_frame {
                // all frames of current area are used, switch to next area
                self.choose_next_area();
            } else if (start_frame >= self.kernel_start && start_frame <= self.kernel_end)
                    || (end_frame >= self.kernel_start && end_frame <= self.kernel_end) {
                // `frame` is used by the kernel
                self.next_free_frame = Frame {
                    number: self.kernel_end.number + 1
                };
            } else {
                // frame is unused, increment `next_free_frame` and return it
                self.next_free_frame.number += count;
                return Some(start_frame);
            }
            // `frame` was not valid, try it again with the updated `next_free_frame`
            self.allocate_frames(count)
        } else {
            None // no free frames left
        }
    }

    fn deallocate_frames(&mut self, frame: Frame, count: usize) {
        //panic!("AreaFrameAllocator::deallocate_frame: not supported: {:?}", frame);
    }
}
