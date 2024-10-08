// Copyright 2024 Kevin Ludwig
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core::{arch, ptr};

use alloc::{boxed::Box, collections::vec_deque::VecDeque};
use ctx::Context;

use crate::mm;

mod ctx;

pub struct Scheduler {
    tss: Box<mm::sm::TaskStateSegment>,

    context: Context,
    work_queue: VecDeque<Schedulable>,
    work: Option<Schedulable>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            tss: Default::default(),
            context: unsafe { Context::empty() },
            work_queue: Default::default(),
            work: Default::default(),
        }
    }
}

impl Scheduler {
    pub fn get() -> &'static mut Self {
        unsafe {
            let ctx: *mut Self;
            arch::asm!("mov {ctx}, gs:0", ctx = out(reg) ctx);
            &mut *ctx
        }
    }

    pub fn run(&mut self) {
        while let Some(thread) = self.work_queue.pop_front() {
            self.work = Some(thread);
            self.work.as_ref().unwrap().context.swap(&mut self.context);
        }
    }

    pub fn enter(&mut self, startup: bool) {
        let work = self.work.as_mut().unwrap();
        if startup {
            (work.closure.take().unwrap())();
            self.work = None;
            self.context.load();
        } else {
            self.work_queue.push_back(self.work.take().unwrap());
            self.context
                .swap(&mut self.work_queue.back_mut().unwrap().context);
        }
    }

    pub fn queue(&mut self, closure: Box<dyn FnOnce()>) {
        self.work_queue.push_back(Schedulable::new(closure));
    }
}

pub struct Schedulable {
    context: Context,
    closure: Option<Box<dyn FnOnce()>>,
}

impl Schedulable {
    fn new(closure: Box<dyn FnOnce()>) -> Self {
        fn schedulable_entry_point() -> ! {
            Scheduler::get().enter(true);
            unreachable!();
        }

        Self {
            context: Context::new(schedulable_entry_point),
            closure: Some(closure),
        }
    }
}

pub fn run() -> ! {
    fn scheduler_entry_point() -> ! {
        let mut scheduler: Box<Scheduler> = Box::default();
        scheduler.tss.load();
        mm::sm::GS::set(ptr::addr_of!(scheduler) as usize, size_of::<Scheduler>());

        scheduler.queue(Box::new(|| {
            Scheduler::get().queue(Box::new(|| loop {
                log::info!("Inside the second closure");
                Scheduler::get().enter(false);
            }));
            loop {
                log::info!("Inside the closure");
                Scheduler::get().enter(false);
            }
        }));
        scheduler.run();
        panic!("nothing left to do");
    }

    Context::new(scheduler_entry_point).load();
}
