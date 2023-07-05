use core::time;
use gtk::{glib, glib::clone, prelude::*, Application, ApplicationWindow, DrawingArea};
use std::sync::{Arc, Mutex};
use std::thread;

mod grid_graph;
use grid_graph::{CellType::*, Gg};

const APP_ID: &str = "org.gtk_rs.conways_gol";

static BASECOLOR: [f64; 3] = [0.12, 0.12, 0.18];
static FULLCOLOR: [f64; 3] = [0.19, 0.2, 0.27];

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
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
                    true => FullCell,
                    false => EmptyCell,
                };
                graph.set_val(i, j, value);
            } else {
                let value = match count == 3 {
                    true => FullCell,
                    false => EmptyCell,
                };
                graph.set_val(i, j, value);
            }
        }
    }
}

fn draw_grid(cr: &gtk::cairo::Context, w: usize, h: usize) {
    cr.set_line_width(0.5);
    cr.set_source_rgb(FULLCOLOR[0], FULLCOLOR[1], FULLCOLOR[2]);
    for i in 0..w {
        cr.move_to(i as f64 * 1280.0 / w as f64, 0.0);
        cr.line_to(i as f64 * 1280.0 / w as f64, 720.0);
    }
    for i in 0..h {
        cr.move_to(0.0, i as f64 * 720.0 / h as f64);
        cr.line_to(1280.0, i as f64 * 720.0 / h as f64);
    }
    cr.stroke().expect("Couldn't draw grid");
}

fn build_ui(app: &Application) {
    let (tx, rx) = glib::MainContext::channel::<Gg>(glib::PRIORITY_DEFAULT);

    let graph = Gg::new(16 * 10, 9 * 10);
    let graphref = Arc::new(Mutex::new(graph));
    let playing = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let do_draw_grid = Arc::new(std::sync::atomic::AtomicBool::new(true));

    let _t1 = thread::spawn(
        clone!(@strong graphref, @strong playing, @strong tx => move || loop {
            if playing.fetch_or(false, std::sync::atomic::Ordering::SeqCst)
            {
                let mut graph = graphref.lock().unwrap();
                let newgraph = graph.clone();
                compute(&mut graph, &newgraph);
                tx.send(graph.to_owned()).expect("Couldn't send");
            }
            thread::sleep(time::Duration::from_micros(16667));
        }),
    );

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Game of Life")
        .build();
    let canvas = DrawingArea::new();
    canvas.set_size_request(1280, 720);

    canvas.set_draw_func(clone!(@strong do_draw_grid => move |_, cr, _, _| {
        cr.set_line_width(1.0);
        let [bgr, bgg, bgb] = BASECOLOR;
        cr.set_source_rgb(bgr, bgg, bgb);
        cr.paint().expect("Couldn't paint");
        if do_draw_grid.load(std::sync::atomic::Ordering::SeqCst) {
            draw_grid(cr, 160, 90);
        }
    }));

    rx.attach(
        None,
        clone!(@strong canvas, @strong do_draw_grid => move |graph: Gg| {
            canvas.set_draw_func(clone!(@strong do_draw_grid => move |_, cr, _, _| {
                cr.set_line_width(1.0);
                let [bgr, bgg, bgb] = BASECOLOR;
                cr.set_source_rgb(bgr, bgg, bgb);
                cr.paint().expect("Couldn't paint");
                for i in 0..graph.width() {
                    for j in 0..graph.height() {
                        if graph.get_val(i, j) == FullCell {
                            let (w, h) = (1280. / graph.width as f64, 720. / graph.height as f64);
                            cr.rectangle(i as f64 * w, j as f64 * h, w, h);
                            cr.set_source_rgb(FULLCOLOR[0], FULLCOLOR[1], FULLCOLOR[2]);
                            cr.fill().expect("Couldn't fill");
                        }
                    }
                }
                if do_draw_grid.load(std::sync::atomic::Ordering::SeqCst) {
                    draw_grid(cr, 160, 90);
                }
            }));
            canvas.queue_draw();
            glib::Continue(true)
        }),
    );

    let gesture = gtk::GestureClick::new();

    gesture.set_button(gtk::gdk::ffi::GDK_BUTTON_PRIMARY as u32);

    gesture.connect_pressed(
        clone!(@strong graphref, @strong tx, @weak canvas => move |gesture, _, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            let mut graph = graphref.lock().unwrap();
            let (i, j) = (x * graph.width() as f64 / 1280.0, y * graph.height() as f64 / 720.0);
            graph.toggle_val(i as usize, j as usize).expect("Out of Bounds in toggle");
            tx.send(graph.to_owned()).expect("Couldn't send");
        }),
    );
    canvas.add_controller(gesture);

    let evk = gtk::EventControllerKey::new();
    evk.connect_key_pressed(
        clone!(@strong graphref, @strong do_draw_grid, @strong canvas => move |_, _, code, _| {
            if code == 65 {
                playing.fetch_xor(true, std::sync::atomic::Ordering::SeqCst);
            } else if code == 27 {
                let mut graph = graphref.lock().unwrap();
                for cell in graph.iter_mut() {
                    if rand::random() {
                        *cell = FullCell;
                    } else {
                        *cell = EmptyCell;
                    }
                }
                tx.send(graph.to_owned()).expect("Couldn't send");
            } else if code == 42 {
                do_draw_grid.fetch_xor(true, std::sync::atomic::Ordering::SeqCst);
                tx.send(graphref.lock().unwrap().to_owned())
                    .expect("Couldn't send");
            } else if code == 54 {
                let mut graph = graphref.lock().unwrap();
                for cell in graph.iter_mut() {
                    *cell = EmptyCell;
                }
                tx.send(graph.to_owned()).expect("Couldn't send");
            }
            gtk::Inhibit(false)
        }),
    );
    window.add_controller(evk);

    window.set_child(Some(&canvas));
    window.present();
}
