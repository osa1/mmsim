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

fn build_ui(app: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(app);

    // layout: vbox [ drawing_area, hbox [ settings1, settings2 ] ]
    // Settings are grids
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);

    let chart_data_vec: Vec<(u32, u32)> = (0..=200).map(|x| (x, x * x)).collect();
    let chart_data: Rc<RefCell<Vec<(u32, u32)>>> = Rc::new(RefCell::new(chart_data_vec));

    {
        let chart_data_clone = chart_data.clone();
        let drawing_area = gtk::DrawingArea::new();
        drawing_area
            .connect_draw(move |w, cr| drawing_area_on_draw(w, cr, &*chart_data_clone.borrow()));
        vbox.pack_start(&drawing_area, true, true, 5);
    }

    {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 5);

        let mutator_settings = build_mutator_settings_grid();
        let gc_settings = build_gc_settings_grid();

        hbox.pack_start(&mutator_settings, false, true, 0);
        hbox.pack_end(&gc_settings, false, true, 0);

        vbox.pack_end(&hbox, false, true, 5);
    }

    window.set_default_size(500, 500);
    window.add(&vbox);
    window.show_all();
}

fn build_mutator_settings_grid() -> gtk::Widget {
    let grid = gtk::Grid::new();

    let num_calls_label = gtk::Label::new(Some("Number of calls:"));
    let num_calls_entry = gtk::Entry::new();

    let allocation_rate_label = gtk::Label::new(Some("Allocation rate (bytes/call):"));
    let allocation_rate_entry = gtk::Entry::new();

    let live_data_label = gtk::Label::new(Some("Survival rate (%):"));
    let live_data_entry = gtk::Entry::new();

    grid.attach(&num_calls_label, 0, 0, 1, 1);
    grid.attach(&num_calls_entry, 1, 0, 1, 1);
    grid.attach(&allocation_rate_label, 0, 1, 1, 1);
    grid.attach(&allocation_rate_entry, 1, 1, 1, 1);
    grid.attach(&live_data_label, 0, 2, 1, 1);
    grid.attach(&live_data_entry, 1, 2, 1, 1);

    grid.upcast()
}

fn build_gc_settings_grid() -> gtk::Widget {
    let grid = gtk::Grid::new();

    let growth_factor_label = gtk::Label::new(Some("Heap growth factor:"));
    let growth_factor_entry = gtk::Entry::new();

    let small_heap_delta_label = gtk::Label::new(Some("Small heap delta: (bytes)"));
    let small_heap_delta_entry = gtk::Entry::new();

    let max_heap_for_gc_label = gtk::Label::new(Some("Max heap for GC:"));
    let max_heap_for_gc_entry = gtk::Entry::new();

    grid.attach(&growth_factor_label, 0, 0, 1, 1);
    grid.attach(&growth_factor_entry, 1, 0, 1, 1);
    grid.attach(&small_heap_delta_label, 0, 1, 1, 1);
    grid.attach(&small_heap_delta_entry, 1, 1, 1, 1);
    grid.attach(&max_heap_for_gc_label, 0, 2, 1, 1);
    grid.attach(&max_heap_for_gc_entry, 1, 2, 1, 1);

    grid.upcast()
}

fn drawing_area_on_draw(
    _widget: &gtk::DrawingArea,
    cr: &cairo::Context,
    chart_data: &[(u32, u32)],
) -> gtk::Inhibit {
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

    chart
        .draw_series(LineSeries::new(chart_data.iter().copied(), &GREEN))
        .unwrap();

    Inhibit(false)
}
