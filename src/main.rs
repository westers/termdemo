mod app;
mod effect;
mod effects;
mod framebuffer;
mod input;
mod scene;
mod sequencer;
mod transition;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::{App, Mode};
use effects::aurora::Aurora;
use effects::boingball::BoingBall;
use effects::boids::Boids;
use effects::cellular::CellularAutomata;
use effects::copperflag::CopperFlag;
use effects::filledvector::FilledVector;
use effects::fluidsim::FluidSim;
use effects::fractalzoom::FractalZoom;
use effects::lightning::Lightning;
use effects::morph::Morph;
use effects::oscilloscope::Oscilloscope;
use effects::reaction::ReactionDiffusion;
use effects::sinescroller::SineScroller;
use effects::snowfall::Snowfall;
use effects::spirograph::Spirograph;
use effects::truchet::Truchet;
use effects::wolfenstein::Wolfenstein;
use effects::clothsim::ClothSim;
use effects::cubefield::CubeField;
use effects::dottunnel::DotTunnel;
use effects::flowfield::FlowField;
use effects::interference::Interference;
use effects::kefrensbars::KefrensBars;
use effects::lavalamp::LavaLamp;
use effects::lsystem::LSystem;
use effects::neon::Neon;
use effects::parallax::Parallax;
use effects::pendulum::PendulumWave;
use effects::pixelsort::PixelSort;
use effects::rain::Rain;
use effects::sierpinski::Sierpinski;
use effects::terrain::Terrain;
use effects::bumpmapping::BumpMapping;
use effects::copperbars::CopperBars;
use effects::dotsphere::DotSphere;
use effects::fire::Fire;
use effects::fireworks::Fireworks;
use effects::fountain::Fountain;
use effects::galaxy::Galaxy;
use effects::gameoflife::GameOfLife;
use effects::glenz::Glenz;
use effects::kaleidoscope::Kaleidoscope;
use effects::julia::Julia;
use effects::lens::Lens;
use effects::lissajous::Lissajous3D;
use effects::mandelbrot::Mandelbrot;
use effects::matrix::Matrix;
use effects::metaballs::Metaballs;
use effects::moire::Moire;
use effects::plasma::Plasma;
use effects::rasterbars::RasterBars;
use effects::raymarcher::Raymarcher;
use effects::shadebobs::Shadebobs;
use effects::rotozoom::Rotozoom;
use effects::scroller::Scroller;
use effects::starfield::Starfield;
use effects::torusknot::TorusKnot;
use effects::tunnel::Tunnel;
use effects::twister::Twister;
use effects::voronoi::Voronoi;
use effects::voxel::VoxelLandscape;
use effects::water::Water;
use effects::wireframe::Wireframe;
use framebuffer::HalfBlockWidget;
use ui::HudWidget;
use scene::Scene;
use sequencer::Sequencer;
use transition::TransitionKind;

fn main() -> io::Result<()> {
    let interactive = std::env::args().any(|a| a == "-i" || a == "--interactive");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = run(&mut terminal, interactive);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn build_scenes() -> Vec<Scene> {
    vec![
        // ACT 1 — Classic Patterns
        Scene::new(Box::new(Plasma::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(Moire::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(Kaleidoscope::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(Shadebobs::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(CopperBars::new()))
            .with_duration(10.0)
            .with_transition(TransitionKind::WipeDown, 1.5),
        Scene::new(Box::new(RasterBars::new()))
            .with_duration(10.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(CopperFlag::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(KefrensBars::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(Truchet::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(Interference::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        // ACT 2 — Heat & Motion
        Scene::new(Box::new(Fire::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::WipeDown, 1.5),
        Scene::new(Box::new(Twister::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(Tunnel::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Fade, 1.5),
        Scene::new(Box::new(DotTunnel::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(Rotozoom::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(Lightning::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Fade, 1.5),
        Scene::new(Box::new(LavaLamp::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        // ACT 3 — 3D Geometry
        Scene::new(Box::new(Starfield::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Fade, 1.5),
        Scene::new(Box::new(Galaxy::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 2.0),
        Scene::new(Box::new(DotSphere::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(BoingBall::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(FilledVector::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(Morph::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(Glenz::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(Lissajous3D::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(TorusKnot::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 2.0),
        Scene::new(Box::new(Wireframe::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Fade, 1.5),
        Scene::new(Box::new(CubeField::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Fade, 1.5),
        Scene::new(Box::new(Wolfenstein::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Fade, 1.5),
        Scene::new(Box::new(Raymarcher::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 2.0),
        Scene::new(Box::new(Terrain::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 2.0),
        Scene::new(Box::new(VoxelLandscape::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 2.0),
        // ACT 4 — Fractals
        Scene::new(Box::new(Mandelbrot::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 2.0),
        Scene::new(Box::new(Julia::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 2.0),
        Scene::new(Box::new(FractalZoom::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 2.0),
        Scene::new(Box::new(Sierpinski::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 2.0),
        // ACT 5 — Simulations
        Scene::new(Box::new(Metaballs::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Fade, 1.5),
        Scene::new(Box::new(Voronoi::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(ReactionDiffusion::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(FluidSim::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(ClothSim::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(Water::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(Fountain::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Fade, 1.5),
        Scene::new(Box::new(Boids::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(CellularAutomata::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(GameOfLife::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        // ACT 6 — Natural / Atmospheric
        Scene::new(Box::new(Aurora::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Fade, 2.0),
        Scene::new(Box::new(Rain::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(Snowfall::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(Parallax::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 2.0),
        Scene::new(Box::new(LSystem::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(Neon::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Fade, 1.5),
        // ACT 7 — Retro / Text
        Scene::new(Box::new(Lens::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(BumpMapping::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(SineScroller::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(Oscilloscope::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(PendulumWave::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(Spirograph::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Fade, 1.5),
        Scene::new(Box::new(FlowField::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(PixelSort::new()))
            .with_duration(12.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
        Scene::new(Box::new(Matrix::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Fade, 2.0),
        // FINALE
        Scene::new(Box::new(Fireworks::new()))
            .with_duration(14.0)
            .with_transition(TransitionKind::Fade, 2.0),
        Scene::new(Box::new(Scroller::new(
            "63 EFFECTS IN YOUR TERMINAL *** TERMDEMO *** GREETS TO ALL DEMOSCENERS!   ",
        )))
            .with_duration(16.0)
            .with_transition(TransitionKind::WipeLeft, 2.0),
        Scene::new(Box::new(Plasma::with_params(0.6, 2.5)))
            .with_duration(8.0)
            .with_transition(TransitionKind::Dissolve, 1.5),
    ]
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, interactive: bool) -> io::Result<()> {
    let mode = if interactive {
        Mode::Interactive
    } else {
        Mode::AutoPlay
    };

    let scenes = build_scenes();
    let seq = Sequencer::new(scenes, mode == Mode::AutoPlay);
    let mut app = App::new(seq, mode);

    let size = terminal.size()?;
    let fb_width = size.width as u32;
    let fb_height = (size.height as u32) * 2;
    app.init(fb_width, fb_height);

    let target_frame = Duration::from_secs_f64(1.0 / 60.0);

    loop {
        let frame_start = std::time::Instant::now();

        app.handle_input()?;
        if app.should_quit {
            return Ok(());
        }

        // Handle resize (guard against zero-size)
        let new_size = terminal.size()?;
        let new_w = new_size.width as u32;
        let new_h = (new_size.height as u32) * 2;
        if new_w > 0 && new_h > 0 && (new_w != app.fb.width || new_h != app.fb.height) {
            app.resize(new_w, new_h);
        }

        if app.fb.width > 0 && app.fb.height > 0 {
            app.update();

            let show_hud = app.show_hud;
            terminal.draw(|frame| {
                let area = frame.size();
                frame.render_widget(HalfBlockWidget { framebuffer: &app.fb }, area);
                if show_hud {
                    frame.render_widget(HudWidget { app: &app }, area);
                }
            })?;
        }

        // Frame pacing
        let elapsed = frame_start.elapsed();
        if elapsed < target_frame {
            std::thread::sleep(target_frame - elapsed);
        }
    }
}
