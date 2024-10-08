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
mod int;

pub struct Scheduler {
    context: Context,
    runnables: VecDeque<Runnable>,
    running: Option<Runnable>,

    tss: Box<mm::sm::TaskStateSegment>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            context: unsafe { Context::empty() },
            runnables: Default::default(),
            running: Default::default(),
            tss: Default::default(),
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

    fn scheduler_entry() -> ! {
        let mut scheduler: Box<Self> = Box::default();
        unsafe {
            scheduler.tss.load();
            mm::sm::GS::set(ptr::addr_of!(scheduler) as usize, size_of_val(&scheduler));
        }

        while let Some(runnable) = scheduler.runnables.pop_front() {
            scheduler.running = Some(runnable);
            scheduler
                .running
                .as_ref()
                .unwrap()
                .context
                .swap(&mut scheduler.context);
        }
        panic!("nothing left to do");
    }

    fn runnable_entry() -> ! {
        let scheduler = Scheduler::get();
        let runnable = scheduler.running.as_mut().unwrap();
        (runnable.closure.take().unwrap())();
        scheduler.running = None;
        scheduler.context.load();
    }

    pub fn r#yield(&mut self) {
        self.runnables.push_back(self.running.take().unwrap());
        self.context
            .swap(&mut self.runnables.back_mut().unwrap().context);
    }

    pub fn spawn(&mut self, closure: Box<dyn FnOnce()>) {
        self.runnables.push_back(Runnable::new(closure));
    }
}

pub struct Runnable {
    context: Context,
    closure: Option<Box<dyn FnOnce()>>,
}

impl Runnable {
    fn new(closure: Box<dyn FnOnce()>) -> Self {
        Self {
            context: Context::new(Scheduler::runnable_entry),
            closure: Some(closure),
        }
    }
}

pub fn run() -> ! {
    Context::new(Scheduler::scheduler_entry).load();
}
