use super::{ChartItem, ChartType, NextPage, Page, SharedState};
use crate::icons::Icons;
use crate::tournament;
use anyhow::Result;
use macroquad::prelude::*;
use prpr::{
    ext::{semi_black, RectExt, BLACK_TEXTURE},
    scene::{request_input, return_input, show_error, show_message, take_input},
    task::Task,
    ui::{DRectButton, Ui},
};
use std::borrow::Cow;
use std::sync::Arc;

pub struct VerifyPage {
    icons: Arc<Icons>,
    expected_code: String,
    step: usize,
    input_text: String,
    input_btn: DRectButton,
    confirm_btn: DRectButton,
    import_task: Option<Task<Result<String>>>,
    next_page: Option<NextPage>,
}

impl VerifyPage {
    pub fn new(icons: Arc<Icons>, expected_code: String, step: usize) -> Self {
        Self {
            icons,
            expected_code,
            step,
            input_text: String::new(),
            input_btn: DRectButton::new(),
            confirm_btn: DRectButton::new(),
            import_task: None,
            next_page: None,
        }
    }
}

impl Page for VerifyPage {
    fn label(&self) -> Cow<'static, str> {
        "验证".into()
    }

    fn is_blocking(&self) -> bool {
        true
    }

    fn touch(&mut self, touch: &Touch, s: &mut SharedState) -> Result<bool> {
        if self.import_task.is_some() {
            return Ok(true);
        }
        if self.input_btn.touch(touch, s.t) {
            request_input("tournament_code", inputbox::InputBox::new().default_text(&self.input_text));
            return Ok(true);
        }
        if self.confirm_btn.touch(touch, s.t) {
            if self.input_text.trim() != self.expected_code {
                show_message("验证码错误").error();
                return Ok(true);
            }
            let pez_name = match self.step {
                0 => "pez2.pez",
                1 => "pez3.pez",
                _ => return Ok(true),
            };
            self.import_task = Some(Task::new(tournament::import_pez(pez_name)));
            return Ok(true);
        }
        Ok(false)
    }

    fn update(&mut self, _s: &mut SharedState) -> Result<()> {
        if let Some((id, text)) = take_input() {
            if id == "tournament_code" {
                self.input_text = text;
            } else {
                return_input(id, text);
            }
        }
        if let Some(task) = &mut self.import_task {
            if let Some(res) = task.take() {
                self.import_task = None;
                match res {
                    Err(err) => {
                        show_error(err.context("导入下一首失败"));
                    }
                    Ok(local_path) => {
                        let data = crate::get_data();
                        let chart = data.charts.iter().find(|it| it.local_path == local_path).unwrap();
                        let item = ChartItem {
                            info: chart.info.clone(),
                            local_path: Some(local_path.clone()),
                            illu: crate::page::local_illustration(local_path.clone(), BLACK_TEXTURE.clone(), false),
                            chart_type: ChartType::Imported,
                        };
                        *tournament::PENDING_SONG.lock().unwrap() = Some((item, local_path));
                        
                        tournament::advance();
                    }
                }
            }
        }
        Ok(())
    }

    fn render(&mut self, ui: &mut Ui, s: &mut SharedState) -> Result<()> {
        ui.fill_rect(ui.screen_rect(), semi_black(0.8));
        let wr = Ui::dialog_rect().nonuniform_feather(0.2, 0.1);
        ui.fill_path(&wr.rounded(0.02), ui.background());
        ui.text("请输入验证码").pos(wr.center().x, wr.y + 0.06).anchor(0.5, 0.).size(0.8).draw();
        let r = Rect::new(wr.x + 0.04, wr.y + 0.14, wr.w - 0.08, 0.1);
        self.input_btn.render_input(ui, r, s.t, &self.input_text, "", 0.6);
        let r = Rect::new(wr.x + 0.04, wr.bottom() - 0.14, wr.w - 0.08, 0.1);
        self.confirm_btn.render_text(ui, r, s.t, "确认", 0.6, true);
        if self.import_task.is_some() {
            ui.full_loading_simple(s.t);
        }
        Ok(())
    }

    fn next_page(&mut self) -> NextPage {
        self.next_page.take().unwrap_or_default()
    }
}