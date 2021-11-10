mod graphviz;

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

#[derive(Debug, Clone)]
struct RuntimeConfig {
    num_calls: u32,

    // bytes/call
    allocation_rate: u32,

    // Percentage of newly allocated objects surviving a call
    survival_rate: u32,

    growth_factor: f64,
    small_heap_delta: u64,
    max_hp_for_gc: u64,
}

fn build_ui(app: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(app);

    // layout: vbox [ drawing_area, hbox [ settings1, settings2 ] ]
    // Settings are grids
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);

    let runtime_config = Rc::new(RefCell::new(RuntimeConfig {
        num_calls: 1000,
        allocation_rate: 1000,
        survival_rate: 100,
        growth_factor: 1.5f64,
        small_heap_delta: 10 * 1024 * 1024,    // 10 MiB
        max_hp_for_gc: 1 * 1024 * 1024 * 1024, // 1 GiB
    }));

    let drawing_area = gtk::DrawingArea::new();

    {
        let runtime_config_ = runtime_config.clone();
        drawing_area
            .connect_draw(move |w, cr| drawing_area_on_draw(w, cr, &*runtime_config_.borrow()));
        vbox.pack_start(&drawing_area, true, true, 5);
    }

    {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 5);

        let mutator_settings =
            build_mutator_settings_grid(&runtime_config, drawing_area.clone().upcast());
        let gc_settings = build_gc_settings_grid(&runtime_config, drawing_area.upcast());

        hbox.pack_start(&mutator_settings, false, true, 0);
        hbox.pack_end(&gc_settings, false, true, 0);

        vbox.pack_end(&hbox, false, true, 5);
    }

    window.set_default_size(500, 500);
    window.add(&vbox);
    window.show_all();
}

fn build_mutator_settings_grid(
    runtime_config: &Rc<RefCell<RuntimeConfig>>,
    drawing_area: gtk::Widget,
) -> gtk::Widget {
    let grid = gtk::Grid::new();

    let num_calls_label = gtk::Label::new(Some("Number of calls:"));
    num_calls_label.set_halign(gtk::Align::End);

    let num_calls_entry = gtk::Entry::new();
    num_calls_entry.set_text(&runtime_config.borrow().num_calls.to_string());

    {
        let runtime_config_ = runtime_config.clone();
        let drawing_area_ = drawing_area.clone();
        num_calls_entry.connect_activate(move |entry| {
            let value = entry.text().to_string();
            match str::parse::<u32>(&value) {
                Ok(num_calls) => {
                    runtime_config_.borrow_mut().num_calls = num_calls;
                    drawing_area_.queue_draw();
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
        let drawing_area_ = drawing_area.clone();
        allocation_rate_entry.connect_activate(move |entry| {
            let value = entry.text().to_string();
            match str::parse::<u32>(&value) {
                Ok(allocation_rate) => {
                    runtime_config_.borrow_mut().allocation_rate = allocation_rate;
                    drawing_area_.queue_draw();
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
        let drawing_area_ = drawing_area.clone();
        survival_rate_entry.connect_activate(move |entry| {
            let value = entry.text().to_string();
            match str::parse::<u32>(&value) {
                Ok(survival_rate) => {
                    if survival_rate > 100 {
                        println!("Survival rate needs to be in range 0-100");
                    } else {
                        runtime_config_.borrow_mut().survival_rate = survival_rate;
                        drawing_area_.queue_draw();
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
    drawing_area: gtk::Widget,
) -> gtk::Widget {
    let grid = gtk::Grid::new();

    let growth_factor_label = gtk::Label::new(Some("Heap growth factor:"));
    growth_factor_label.set_halign(gtk::Align::End);

    let growth_factor_entry = gtk::Entry::new();
    growth_factor_entry.set_text(&runtime_config.borrow().growth_factor.to_string());

    {
        let runtime_config_ = runtime_config.clone();
        let drawing_area_ = drawing_area.clone();
        growth_factor_entry.connect_activate(move |entry| {
            let value = entry.text().to_string();
            match str::parse::<f64>(&value) {
                Ok(growth_factor) => {
                    runtime_config_.borrow_mut().growth_factor = growth_factor;
                    drawing_area_.queue_draw();
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
        let drawing_area_ = drawing_area.clone();
        small_heap_delta_entry.connect_activate(move |entry| {
            let value = entry.text().to_string();
            match str::parse::<u64>(&value) {
                Ok(small_heap_delta) => {
                    runtime_config_.borrow_mut().small_heap_delta = small_heap_delta;
                    drawing_area_.queue_draw();
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
        let drawing_area_ = drawing_area.clone();
        max_hp_for_gc_entry.connect_activate(move |entry| {
            let value = entry.text().to_string();
            match str::parse::<u64>(&value) {
                Ok(max_hp_for_gc) => {
                    runtime_config_.borrow_mut().max_hp_for_gc = max_hp_for_gc;
                    drawing_area_.queue_draw();
                }
                Err(parse_err) => println!("Unable to parse small heap delta: {:?}", parse_err),
            }
        });
    }

    grid.attach(&growth_factor_label, 0, 0, 1, 1);
    grid.attach(&growth_factor_entry, 1, 0, 1, 1);
    grid.attach(&small_heap_delta_label, 0, 1, 1, 1);
    grid.attach(&small_heap_delta_entry, 1, 1, 1, 1);
    grid.attach(&max_hp_for_gc_label, 0, 2, 1, 1);
    grid.attach(&max_hp_for_gc_entry, 1, 2, 1, 1);

    grid.upcast()
}

fn drawing_area_on_draw(
    _widget: &gtk::DrawingArea,
    cr: &cairo::Context,
    runtime_config: &RuntimeConfig,
) -> gtk::Inhibit {
    println!("Drawing chart...");

    Inhibit(false)
}

fn generate_chart(config: &RuntimeConfig) -> Vec<(u32, u32)> {
    let RuntimeConfig {
        num_calls,
        allocation_rate,
        survival_rate,
        growth_factor,
        small_heap_delta,
        max_hp_for_gc,
    } = config;

    let mut points: Vec<(u32, u32)> = Vec::with_capacity(usize::try_from(*num_calls).unwrap() * 3);

    let mut hp: u32 = 0;

    for x in 1..=*num_calls {
        let last_hp = hp;

        hp += *allocation_rate;
        points.push((3 * x, u32::try_from(hp).unwrap()));

        let heap_limit = min(
            max(
                (f64::from(last_hp) * *growth_factor) as u64,
                u64::from(last_hp) + small_heap_delta,
            ),
            *max_hp_for_gc,
        );

        if heap_limit > u64::from(hp) {
            let new_live_data =
                (f64::from(*allocation_rate) * f64::from(*survival_rate) / 100f64) as u64;
            let total_live_data = u64::from(last_hp) + new_live_data;
            let hp_during_copying_gc = hp + total_live_data as u32;
            points.push((3 * x + 1, hp_during_copying_gc));

            let hp_after_copying_gc = total_live_data;
            points.push((3 * x + 2, hp_after_copying_gc as u32));
        } else {
            points.push((3 * x + 1, hp));
            points.push((3 * x + 2, hp));
        }
    }

    points
}
