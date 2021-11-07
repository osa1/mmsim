use std::cell::RefCell;
use std::rc::Rc;

use gio::prelude::*;
use gtk::prelude::*;

use plotters::prelude::*;
use plotters_cairo::CairoBackend;

fn main() {
    let application = gtk::Application::new(
        Some("io.github.plotters-rs.plotters-gtk-test"),
        Default::default(),
    )
    .expect("Initialization failed");

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&std::env::args().collect::<Vec<_>>());
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

    drawing_area.connect_draw(drawing_area_on_draw);
    vbox.pack_start(&drawing_area, true, true, 5);

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
    let num_calls_entry = gtk::Entry::new();
    num_calls_entry.set_text(&runtime_config.borrow().num_calls.to_string());

    {
        let runtime_config_ = runtime_config.clone();
        let drawing_area_ = drawing_area.clone();
        num_calls_entry.connect_activate(move |entry| {
            let value = entry.get_text().to_string();
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
    let allocation_rate_entry = gtk::Entry::new();
    allocation_rate_entry.set_text(&runtime_config.borrow().allocation_rate.to_string());

    {
        let runtime_config_ = runtime_config.clone();
        let drawing_area_ = drawing_area.clone();
        allocation_rate_entry.connect_activate(move |entry| {
            let value = entry.get_text().to_string();
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
    let survival_rate_entry = gtk::Entry::new();
    survival_rate_entry.set_text(&runtime_config.borrow().survival_rate.to_string());

    {
        let runtime_config_ = runtime_config.clone();
        let drawing_area_ = drawing_area.clone();
        survival_rate_entry.connect_activate(move |entry| {
            let value = entry.get_text().to_string();
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
    let growth_factor_entry = gtk::Entry::new();
    growth_factor_entry.set_text(&runtime_config.borrow().growth_factor.to_string());

    {
        let runtime_config_ = runtime_config.clone();
        let drawing_area_ = drawing_area.clone();
        growth_factor_entry.connect_activate(move |entry| {
            let value = entry.get_text().to_string();
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
    let small_heap_delta_entry = gtk::Entry::new();
    small_heap_delta_entry.set_text(&runtime_config.borrow().small_heap_delta.to_string());

    {
        let runtime_config_ = runtime_config.clone();
        let drawing_area_ = drawing_area.clone();
        small_heap_delta_entry.connect_activate(move |entry| {
            let value = entry.get_text().to_string();
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
    let max_hp_for_gc_entry = gtk::Entry::new();
    max_hp_for_gc_entry.set_text(&runtime_config.borrow().max_hp_for_gc.to_string());

    {
        let runtime_config_ = runtime_config.clone();
        let drawing_area_ = drawing_area.clone();
        max_hp_for_gc_entry.connect_activate(move |entry| {
            let value = entry.get_text().to_string();
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

fn drawing_area_on_draw(_widget: &gtk::DrawingArea, cr: &cairo::Context) -> gtk::Inhibit {
    println!("Drawing chart...");

    let root = CairoBackend::new(cr, (500, 500))
        .unwrap()
        .into_drawing_area();

    root.fill(&WHITE).unwrap();
    let root = root.margin(25, 25, 25, 25);

    let mut chart = ChartBuilder::on(&root)
        .caption("This is a test", ("monospace", 20))
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(0u32..100u32, 0u32..100u32)
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    let chart_data_vec: Vec<(u32, u32)> = (0..=200).map(|x| (x, x * x)).collect();
    let chart_data = &chart_data_vec;

    chart
        .draw_series(LineSeries::new(chart_data.iter().copied(), &GREEN))
        .unwrap();

    Inhibit(false)
}
