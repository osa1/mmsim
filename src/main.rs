mod graphviz;
mod image_widget;

use image_widget::Image;

use std::cell::RefCell;
use std::cmp::{max, min};
use std::rc::Rc;

use gio::prelude::*;
use gtk::prelude::*;

fn main() {
    let application = gtk::Application::new(Some("net.osa1.mmsim"), Default::default());

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run();
}

#[derive(Debug, Clone, Copy)]
enum GcStrategy {
    MarkCompact,
    Copying,
}

#[derive(Debug, Clone, Copy)]
enum Scheduler {
    Old,
    New,
}

#[derive(Debug, Clone, Copy)]
struct RuntimeConfig {
    gc_strategy: GcStrategy,
    scheduler: Scheduler,

    num_calls: u32,

    // bytes/call
    allocation_rate: u32,

    // Percentage of newly allocated objects surviving a call
    survival_rate: u32,

    growth_factor: f64,
    small_heap_delta: u64,
    max_hp_for_gc: u64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            gc_strategy: GcStrategy::Copying,
            scheduler: Scheduler::Old,
            num_calls: 100_000,
            allocation_rate: 100_000,
            survival_rate: 50,
            growth_factor: 1.5f64,
            small_heap_delta: 10 * 1024 * 1024,    // 10 MiB
            max_hp_for_gc: 2 * 1024 * 1024 * 1024, // 1 GiB
        }
    }
}

fn build_ui(app: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(app);

    // layout: vbox [ image, hbox [ settings1, settings2 ] ]
    // Settings are grids
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);

    let runtime_config = Rc::new(RefCell::new(RuntimeConfig::default()));

    let image = Image::new();
    vbox.pack_start(image.widget(), true, true, 0);

    {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 5);

        let mutator_settings = build_mutator_settings_grid(&runtime_config, &image);
        let gc_settings = build_gc_settings_grid(&runtime_config, &image);

        hbox.pack_start(&mutator_settings, false, true, 0);
        hbox.pack_end(&gc_settings, false, true, 0);

        vbox.pack_end(&hbox, false, true, 5);
    }

    window.set_default_size(500, 500);
    window.add(&vbox);
    window.show_all();

    update(*runtime_config.borrow(), &image);
}

fn build_mutator_settings_grid(
    runtime_config: &Rc<RefCell<RuntimeConfig>>,
    image: &Image,
) -> gtk::Widget {
    let grid = gtk::Grid::new();

    let num_calls_label = gtk::Label::new(Some("Number of calls:"));
    num_calls_label.set_halign(gtk::Align::End);

    let num_calls_entry = gtk::Entry::new();
    num_calls_entry.set_text(&runtime_config.borrow().num_calls.to_string());

    {
        let runtime_config_ = runtime_config.clone();
        let image_ = image.clone();
        num_calls_entry.connect_activate(move |entry| {
            let value = entry.text().to_string();
            match str::parse::<u32>(&value) {
                Ok(num_calls) => {
                    runtime_config_.borrow_mut().num_calls = num_calls;
                    update(*runtime_config_.borrow(), &image_);
                }
                Err(parse_err) => println!("Unable to parse number of calls: {:?}", parse_err),
            }
        });
    }

    let allocation_rate_label = gtk::Label::new(Some("Allocation rate (bytes/call):"));
    allocation_rate_label.set_halign(gtk::Align::End);

    let allocation_rate_entry = gtk::Entry::new();
    allocation_rate_entry.set_text(&runtime_config.borrow().allocation_rate.to_string());

    {
        let runtime_config_ = runtime_config.clone();
        let image_ = image.clone();
        allocation_rate_entry.connect_activate(move |entry| {
            let value = entry.text().to_string();
            match str::parse::<u32>(&value) {
                Ok(allocation_rate) => {
                    runtime_config_.borrow_mut().allocation_rate = allocation_rate;
                    update(*runtime_config_.borrow(), &image_);
                }
                Err(parse_err) => println!("Unable to parse allocation rate: {:?}", parse_err),
            }
        });
    }

    let survival_rate_label = gtk::Label::new(Some("Survival rate (%):"));
    survival_rate_label.set_halign(gtk::Align::End);

    let survival_rate_entry = gtk::Entry::new();
    survival_rate_entry.set_text(&runtime_config.borrow().survival_rate.to_string());

    {
        let runtime_config_ = runtime_config.clone();
        let image_ = image.clone();
        survival_rate_entry.connect_activate(move |entry| {
            let value = entry.text().to_string();
            match str::parse::<u32>(&value) {
                Ok(survival_rate) => {
                    if survival_rate > 100 {
                        println!("Survival rate needs to be in range 0-100");
                    } else {
                        runtime_config_.borrow_mut().survival_rate = survival_rate;
                        update(*runtime_config_.borrow(), &image_);
                    }
                }
                Err(parse_err) => println!("Unable to parse allocation rate: {:?}", parse_err),
            }
        });
    }

    grid.attach(&num_calls_label, 0, 0, 1, 1);
    grid.attach(&num_calls_entry, 1, 0, 1, 1);
    grid.attach(&allocation_rate_label, 0, 1, 1, 1);
    grid.attach(&allocation_rate_entry, 1, 1, 1, 1);
    grid.attach(&survival_rate_label, 0, 2, 1, 1);
    grid.attach(&survival_rate_entry, 1, 2, 1, 1);

    grid.upcast()
}

fn build_gc_settings_grid(
    runtime_config: &Rc<RefCell<RuntimeConfig>>,
    image: &Image,
) -> gtk::Widget {
    let grid = gtk::Grid::new();

    let gc_strategy_label = gtk::Label::new(Some("GC strategy"));

    let gc_strategies_combo = {
        let gc_strategies_list = gtk::ListStore::new(&[glib::types::Type::STRING]);
        gc_strategies_list.set(&gc_strategies_list.append(), &[(0, &"Copying".to_value())]);
        gc_strategies_list.set(
            &gc_strategies_list.append(),
            &[(0, &"Mark-compact".to_value())],
        );

        gtk::ComboBox::with_model_and_entry(&gc_strategies_list)
    };

    gc_strategies_combo.set_entry_text_column(0);
    gc_strategies_combo.set_active(Some(0));

    {
        let runtime_config_ = runtime_config.clone();
        let image_ = image.clone();
        gc_strategies_combo.connect_changed(move |combo| match combo.active() {
            Some(0) => {
                runtime_config_.borrow_mut().gc_strategy = GcStrategy::Copying;
                update(*runtime_config_.borrow(), &image_);
            }
            Some(1) => {
                runtime_config_.borrow_mut().gc_strategy = GcStrategy::MarkCompact;
                update(*runtime_config_.borrow(), &image_);
            }
            _ => panic!(),
        });
    }

    let scheduler_label = gtk::Label::new(Some("Scheduler"));

    let scheduler_combo = {
        let scheduler_list = gtk::ListStore::new(&[glib::types::Type::STRING]);
        scheduler_list.set(&scheduler_list.append(), &[(0, &"Old".to_value())]);
        scheduler_list.set(&scheduler_list.append(), &[(0, &"New".to_value())]);

        gtk::ComboBox::with_model_and_entry(&scheduler_list)
    };

    scheduler_combo.set_entry_text_column(0);
    scheduler_combo.set_active(Some(0));

    {
        let runtime_config_ = runtime_config.clone();
        let image_ = image.clone();
        scheduler_combo.connect_changed(move |combo| match combo.active() {
            Some(0) => {
                runtime_config_.borrow_mut().scheduler = Scheduler::Old;
                update(*runtime_config_.borrow(), &image_);
            }
            Some(1) => {
                runtime_config_.borrow_mut().scheduler = Scheduler::New;
                update(*runtime_config_.borrow(), &image_);
            }
            _ => panic!(),
        });
    }

    let growth_factor_label = gtk::Label::new(Some("Heap growth factor:"));
    growth_factor_label.set_halign(gtk::Align::End);

    let growth_factor_entry = gtk::Entry::new();
    growth_factor_entry.set_text(&runtime_config.borrow().growth_factor.to_string());

    {
        let runtime_config_ = runtime_config.clone();
        let image_ = image.clone();
        growth_factor_entry.connect_activate(move |entry| {
            let value = entry.text().to_string();
            match str::parse::<f64>(&value) {
                Ok(growth_factor) => {
                    runtime_config_.borrow_mut().growth_factor = growth_factor;
                    update(*runtime_config_.borrow(), &image_);
                }
                Err(parse_err) => println!("Unable to parse growth factor: {:?}", parse_err),
            }
        });
    }

    let small_heap_delta_label = gtk::Label::new(Some("Small heap delta: (bytes)"));
    small_heap_delta_label.set_halign(gtk::Align::End);

    let small_heap_delta_entry = gtk::Entry::new();
    small_heap_delta_entry.set_text(&runtime_config.borrow().small_heap_delta.to_string());

    {
        let runtime_config_ = runtime_config.clone();
        let image_ = image.clone();
        small_heap_delta_entry.connect_activate(move |entry| {
            let value = entry.text().to_string();
            match str::parse::<u64>(&value) {
                Ok(small_heap_delta) => {
                    runtime_config_.borrow_mut().small_heap_delta = small_heap_delta;
                    update(*runtime_config_.borrow(), &image_);
                }
                Err(parse_err) => println!("Unable to parse small heap delta: {:?}", parse_err),
            }
        });
    }

    let max_hp_for_gc_label = gtk::Label::new(Some("Max hp for GC:"));
    max_hp_for_gc_label.set_halign(gtk::Align::End);

    let max_hp_for_gc_entry = gtk::Entry::new();
    max_hp_for_gc_entry.set_text(&runtime_config.borrow().max_hp_for_gc.to_string());

    {
        let runtime_config_ = runtime_config.clone();
        let image_ = image.clone();
        max_hp_for_gc_entry.connect_activate(move |entry| {
            let value = entry.text().to_string();
            match str::parse::<u64>(&value) {
                Ok(max_hp_for_gc) => {
                    runtime_config_.borrow_mut().max_hp_for_gc = max_hp_for_gc;
                    update(*runtime_config_.borrow(), &image_);
                }
                Err(parse_err) => println!("Unable to parse small heap delta: {:?}", parse_err),
            }
        });
    }

    grid.attach(&gc_strategy_label, 0, 0, 1, 1);
    grid.attach(&gc_strategies_combo, 1, 0, 1, 1);
    grid.attach(&scheduler_label, 0, 1, 1, 1);
    grid.attach(&scheduler_combo, 1, 1, 1, 1);
    grid.attach(&growth_factor_label, 0, 2, 1, 1);
    grid.attach(&growth_factor_entry, 1, 2, 1, 1);
    grid.attach(&small_heap_delta_label, 0, 3, 1, 1);
    grid.attach(&small_heap_delta_entry, 1, 3, 1, 1);
    grid.attach(&max_hp_for_gc_label, 0, 4, 1, 1);
    grid.attach(&max_hp_for_gc_entry, 1, 4, 1, 1);

    grid.upcast()
}

fn update(config: RuntimeConfig, widget: &Image) {
    let Points { hp, high_water } = generate_points(config);
    match graphviz::render(&hp, &high_water) {
        Err(err) => {
            println!("grpahviz error: {:?}", err);
        }
        Ok(image_path) => {
            widget.set_image(&*image_path);
        }
    }
}

#[derive(Debug)]
struct Points {
    hp: Vec<u32>,
    high_water: Vec<u32>,
}

fn generate_points(config: RuntimeConfig) -> Points {
    let RuntimeConfig {
        gc_strategy,
        scheduler,
        num_calls,
        allocation_rate,
        survival_rate,
        growth_factor,
        small_heap_delta,
        max_hp_for_gc,
    } = config;

    let mut hp: Vec<u32> = Vec::with_capacity(config.num_calls as usize);
    hp.push(0);

    let mut high_water: Vec<u32> = Vec::with_capacity(config.num_calls as usize);
    high_water.push(0);

    // Heap pointer after last gc
    let mut last_hp: u32 = 0;

    // Current heap pointer
    let mut hp_: u32 = 0;

    // High water mark for Wasm memory
    // NB. This is in bytes, not rounded up to Wasm page size
    let mut last_high_water: u32 = 0;

    // Number of gcs
    #[allow(unused)]
    let mut num_gcs: u32 = 0;

    // Number of calls made so far
    #[allow(unused)]
    let mut n_calls = 0;

    for _ in 0..num_calls {
        n_calls += 1;

        const COPYING_GC_MAX_LIVE: u64 = 2 * 1024 * 1024 * 1024; // 2 GiB

        // Mark stack ignored. Max. bitmap can be 130,150,524 bytes.
        // (x + x / 32 = 4 GiB, x = 4,164,816,771, x/32 = 130,150,524)
        const MARK_COMPACT_GC_MAX_LIVE: u64 = 4_164_816_771;
        const MARK_COMPACT_GC_MAX_BITMAP_SIZE: u32 = 130_150_524;

        let heap_limit = match scheduler {
            Scheduler::Old => min(
                max(
                    (f64::from(last_hp) * growth_factor) as u64,
                    u64::from(last_hp) + small_heap_delta,
                ),
                max_hp_for_gc,
            ),
            Scheduler::New => {
                let max_live = match gc_strategy {
                    GcStrategy::MarkCompact => MARK_COMPACT_GC_MAX_LIVE,
                    GcStrategy::Copying => COPYING_GC_MAX_LIVE,
                };
                min(
                    (f64::from(last_hp) * growth_factor) as u64,
                    (u64::from(last_hp) + max_live) / 2,
                )
            }
        };

        hp_ += allocation_rate;

        if u64::from(hp_) >= heap_limit {
            num_gcs += 1;

            // New allocations since last GC
            let new_allocs = hp_ - last_hp;

            // Live data since last GC
            let new_live = (f64::from(new_allocs) * f64::from(survival_rate) / 100f64) as u32;

            // Do GC
            match gc_strategy {
                GcStrategy::MarkCompact => {
                    // Mark-compact GC only allocates a bitmap. Mark stack size is ignored.
                    match hp_.checked_add(MARK_COMPACT_GC_MAX_BITMAP_SIZE) {
                        Some(high_water) => last_high_water = max(last_high_water, high_water),
                        None => break,
                    }
                }
                GcStrategy::Copying => {
                    // Copying GC copies the entire live heap to another space and then back
                    let copied = last_hp + new_live;
                    match hp_.checked_add(copied) {
                        Some(high_water) => last_high_water = max(last_high_water, high_water),
                        None => break,
                    }
                }
            }

            hp_ = last_hp + new_live;
            high_water.push(last_high_water);
            hp.push(hp_);
            last_hp = hp_;

            // println!("GC=YES, hp={}, high water={}", hp_, last_high_water);
        } else {
            // No GC
            last_high_water = max(last_high_water, hp_);
            high_water.push(last_high_water);
            hp.push(hp_);

            // println!("GC=NO, hp={}, high water={}", hp_, last_high_water);
        }
    }

    // println!("GCs={}, total_calls={}", num_gcs, n_calls);

    Points { hp, high_water }
}
