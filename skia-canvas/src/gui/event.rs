use skia_safe::{Matrix, Color};
use serde::Serialize;
use serde_json::json;
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};
use winit::{
  dpi::{LogicalSize, LogicalPosition, PhysicalPosition},
  event_loop::{ControlFlow},
  event::{WindowEvent, ElementState,  KeyboardInput, VirtualKeyCode, ModifiersState, MouseButton, MouseScrollDelta},
  window::{CursorIcon, WindowId},
};

use crate::context::page::Page;
use super::window::{WindowSpec, Fit};

#[derive(Debug, Clone)]
pub enum CanvasEvent{
  // app api
  Open(WindowSpec, Page),
  Close(String),
  FrameRate(u64),
  Quit,

  // app -> window
  Page(Page),

  // window -> app
  Transform(WindowId, Option<Matrix>),
  InFullscreen(WindowId, bool),

  // cadence triggers
  Render,

  // script -> window
  Title(String),
  Fullscreen(bool),
  Visible(bool),
  Cursor(Option<CursorIcon>),
  Background(Color),
  Fit(Fit),
  Position(LogicalPosition<i32>),
  Size(LogicalSize<u32>),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UiEvent{
  #[allow(non_snake_case)]
  Wheel{deltaX:f32, deltaY:f32},
  Move{left:f32, top:f32},
  Keyboard{event:String, key:VirtualKeyCode, code:u32, repeat:bool},
  Input(char),
  Mouse(String),
  Focus(bool),
  Resize(LogicalSize<u32>),
  Fullscreen(bool),
}


#[derive(Debug)]
pub struct Sieve{
  dpr: f64,
  queue: Vec<UiEvent>,
  key_modifiers: ModifiersState,
  key_repeats: HashMap<VirtualKeyCode, i32>,
  mouse_point: PhysicalPosition::<f64>,
  mouse_button: Option<u16>,
  mouse_transform: Matrix,
}

impl Sieve{
  pub fn new(dpr:f64) -> Self {
    Sieve{
      dpr,
      queue: vec![],
      key_modifiers: ModifiersState::empty(),
      key_repeats: HashMap::new(),
      mouse_point: PhysicalPosition::default(),
      mouse_button: None,
      mouse_transform: Matrix::new_identity(),
    }
  }

  pub fn use_transform(&mut self, matrix:Matrix){
    self.mouse_transform = matrix;
  }

  pub fn go_fullscreen(&mut self, is_full:bool){
    self.queue.push(UiEvent::Fullscreen(is_full));
    self.key_repeats.clear(); // keyups don't get delivered during the transition apparently?
  }

  pub fn capture(&mut self, event:&WindowEvent){
    match event{
      WindowEvent::Moved(physical_pt) => {
        let LogicalPosition{x, y} = physical_pt.to_logical(self.dpr);
        self.queue.push(UiEvent::Move{left:x, top:y});
      }

      WindowEvent::Resized(physical_size) => {
        let logical_size = LogicalSize::from_physical(*physical_size, self.dpr);
        self.queue.push(UiEvent::Resize(logical_size));
      }

      WindowEvent::Focused(in_focus) => {
        self.queue.push(UiEvent::Focus(*in_focus));
      }

      WindowEvent::ModifiersChanged(state) => {
        self.key_modifiers = *state;
      }

      WindowEvent::ReceivedCharacter(character) => {
        self.queue.push(UiEvent::Input(*character));
      }

      WindowEvent::CursorEntered{..} => {
        let mouse_event = "mouseenter".to_string();
        self.queue.push(UiEvent::Mouse(mouse_event));
      }

      WindowEvent::CursorLeft{..} => {
        let mouse_event = "mouseleave".to_string();
        self.queue.push(UiEvent::Mouse(mouse_event));
      }

      WindowEvent::CursorMoved{position, ..} => {
        if *position != self.mouse_point{
          self.mouse_point = *position;
          self.queue.push(UiEvent::Mouse("mousemove".to_string()));
        }
      }

      WindowEvent::MouseWheel{delta, ..} => {
        let LogicalPosition{x, y} = match delta {
          MouseScrollDelta::PixelDelta(physical_pt) => {
            LogicalPosition::from_physical(*physical_pt, self.dpr)
          },
          MouseScrollDelta::LineDelta(h, v) => {
            LogicalPosition{x:*h as f32, y:*v as f32}
          }
        };
        self.queue.push(UiEvent::Wheel{deltaX:x, deltaY:y});
      }

      WindowEvent::MouseInput{state, button, ..} => {
        let mouse_event = match state {
          ElementState::Pressed => "mousedown",
          ElementState::Released => "mouseup"
        }.to_string();

        self.mouse_button = match button {
          MouseButton::Left => Some(0),
          MouseButton::Middle => Some(1),
          MouseButton::Right => Some(2),
          MouseButton::Other(num) => Some(*num)
        };
        self.queue.push(UiEvent::Mouse(mouse_event));
      }

      WindowEvent::KeyboardInput { input:
        KeyboardInput { scancode, state, virtual_keycode: Some(keycode), ..}, ..
      } => {
        let (event_type, count) = match state{
          ElementState::Pressed => {
            let count = self.key_repeats.entry(*keycode).or_insert(-1);
            *count += 1;
            ("keydown", *count)
          },
          ElementState::Released => {
            self.key_repeats.remove(&keycode);
            ("keyup", 0)
          }
        };

        if event_type == "keyup" || count < 2{
          self.queue.push(UiEvent::Keyboard{
            event: event_type.to_string(),
            key: *keycode,
            code: *scancode,
            repeat: count > 0
          });
        }

      }
      _ => {}
    }
  }

  pub fn serialize(&mut self) -> Option<serde_json::Value>{
    if self.queue.is_empty(){ return None }

    let mut payload: Vec<serde_json::Value> = vec![];
    let mut mouse_events: HashSet<String> = HashSet::new();
    let mut modifiers:Option<ModifiersState> = None;
    let mut last_wheel:Option<&UiEvent> = None;

    for change in &self.queue {
      match change{
        UiEvent::Mouse(event_type) => {
          modifiers = Some(self.key_modifiers);
          mouse_events.insert(event_type.clone());
        }
        UiEvent::Wheel{..} => {
          modifiers = Some(self.key_modifiers);
          last_wheel = Some(&change);
        }
        UiEvent::Input(..) | UiEvent::Keyboard{..} => {
          modifiers = Some(self.key_modifiers);
          payload.push(json!(change));
        }
        _ => payload.push(json!(change))
      }
    }

    if let Some(modfiers) = modifiers {
      payload.insert(0, json!({"modifiers": modifiers}));
    }

    if !mouse_events.is_empty() {
      let viewport_point = LogicalPosition::<f32>::from_physical(self.mouse_point, self.dpr);
      let canvas_point = self.mouse_transform.map_point((viewport_point.x, viewport_point.y));

      payload.push(json!({
        "mouse": {
          "events": mouse_events,
          "button": self.mouse_button,
          "x": canvas_point.x,
          "y": canvas_point.y,
          "pageX": viewport_point.x,
          "pageY": viewport_point.y,
        }
      }));

      if mouse_events.contains("mouseup"){
        self.mouse_button = None;
      }
    }

    if let Some(wheel) = last_wheel{
      payload.push(json!(wheel));
    }

    self.queue.clear();
    Some(json!(payload))
  }

  pub fn is_empty(&self) -> bool {
    self.queue.is_empty()
  }
}

pub struct Cadence{
  rate: u64,
  last: Instant,
  wakeup: Duration,
  render: Duration,
  begun: bool,
}

impl Default for Cadence {
  fn default() -> Self {
    Self{
      rate: 0,
      last: Instant::now(),
      render: Duration::new(0, 0),
      wakeup: Duration::new(0, 0),
      begun: false,
    }
  }
}

impl Cadence{
  pub fn on_startup<F:FnOnce()>(&mut self, init:F){
    if self.begun{ return }
    self.begun = true;
    init();
  }

  pub fn set_frame_rate(&mut self, rate:u64){
    if rate == self.rate{ return }

    let frame_time = 1_000_000_000/rate.max(1);
    let watch_interval = 1_000_000.max(frame_time/10);
    self.render = Duration::from_nanos(frame_time);
    self.wakeup = Duration::from_nanos(frame_time - watch_interval);
    self.rate = rate;
  }

  pub fn on_next_frame<F:Fn()>(&mut self, draw:F) -> ControlFlow{
    if !self.active(){
      return ControlFlow::Wait;
    }

    if self.last.elapsed() >= self.render{
      while self.last < Instant::now() - self.render{
        self.last += self.render
      }
      draw();
    }

    match self.last.elapsed() < self.wakeup {
      true => ControlFlow::WaitUntil(self.last + self.wakeup),
      false => ControlFlow::Poll,
    }
  }

  pub fn active(&self) -> bool{
    self.rate > 0
  }
}

