use catppuccin::Flavour;
use core::time;
use gtk::{glib, glib::clone, prelude::*, Application, ApplicationWindow, DrawingArea};
use std::sync::{Arc, Mutex};
use std::thread;

mod grid_graph;
use grid_graph::{CellType::*, Gg};

const APP_ID: &str = "org.gtk_rs.wallpaper-generator";

static BASECOLOR: [f64; 3] = [0.12, 0.12, 0.18];
static FULLCOLOR: [f64; 3] = [0.19, 0.2, 0.27];

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn one_to_8bit((r, g, b): (u8, u8, u8)) -> [f64; 3] {
    [r as f64 / 255., g as f64 / 255., b as f64 / 255.]
}

fn compute(graph: &mut Gg, state: &Gg) {
    for i in 0..graph.width {
        for j in 0..graph.height {
            let mut count = 0;
            for (ni, nj) in state.neighbors(i, j) {
                if state.get_val(ni, nj) != EmptyCell {
                    count += 1;
                }
            }
            if state.get_val(i, j) != EmptyCell {
                let value = match count == 2 || count == 3 {
                    true => Plant(FULLCOLOR),
                    false => EmptyCell,
                };
                graph.set_val(i, j, value);
            } else {
                let value = match count == 3 {
                    true => Plant(FULLCOLOR),
                    false => EmptyCell,
                };
                graph.set_val(i, j, value);
            }
        }
    }
}

fn build_ui(app: &Application) {
    let (tx, rx) = glib::MainContext::channel::<Gg>(glib::PRIORITY_DEFAULT);

    let graph = Gg::new(16 * 10, 9 * 10);
    let graphref = Arc::new(Mutex::new(graph));
    let cloned_graphref = Arc::clone(&graphref);
    let _t1 = thread::spawn(move || loop {
        {
            let mut graph = cloned_graphref.lock().unwrap();
            let newgraph = graph.clone();
            compute(&mut graph, &newgraph);
            tx.send(graph.clone()).expect("Couldn't send");
        }
        thread::sleep(time::Duration::from_secs(2));
    });

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Wallpaper generator")
        .build();
    let canvas = DrawingArea::new();
    canvas.set_size_request(1280, 720);

    canvas.set_draw_func(move |_, cr, _, _| {
        cr.set_line_width(1.0);
        let [bgr, bgg, bgb] = one_to_8bit(Flavour::Mocha.base().into());
        cr.set_source_rgb(bgr, bgg, bgb);
        cr.paint().expect("Couldn't paint");
    });

    rx.attach(
        None,
        clone!(@strong canvas => move |graph: Gg| {
            canvas.set_draw_func(move |_, cr, _, _| {
                cr.set_line_width(1.0);
                let [bgr, bgg, bgb] = BASECOLOR;
                cr.set_source_rgb(bgr, bgg, bgb);
                cr.paint().expect("Couldn't paint");
                for i in 0..graph.width {
                    for j in 0..graph.height {
                        if let Plant([r, g, b]) = graph.get_val(i, j) {
                            let (w, h) = (1280. / graph.width as f64, 720. / graph.height as f64);
                            cr.rectangle(i as f64 * w, j as f64 * h, w, h);
                            cr.set_source_rgb(r, g, b);
                            cr.fill().expect("Couldn't fill");
                        }
                    }
                }
            });
            canvas.queue_draw();
            glib::Continue(true)
        }),
    );

    let gesture = gtk::GestureClick::new();

    // Set the gestures button to the right mouse button (=3)
    gesture.set_button(gtk::gdk::ffi::GDK_BUTTON_PRIMARY as u32);

    // Assign your handler to an event of the gesture (e.g. the `pressed` event)
    gesture.connect_pressed(clone!(@weak canvas => move |gesture, _, x, y| {
        gesture.set_state(gtk::EventSequenceState::Claimed);
        let mut graph = graphref.lock().unwrap();
        let (i, j) = (x * graph.width() as f64 / 1280.0, y * graph.height() as f64 / 720.0);
        println!("Left mouse button pressed at {}, {}", i, j);
        graph.set_val(i as usize, j as usize, Plant(FULLCOLOR));
        canvas.queue_draw();
    }));
    canvas.add_controller(gesture);

    window.set_child(Some(&canvas));
    window.present();
}
