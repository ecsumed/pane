use ratatui::layout::Layout;
use ratatui::symbols::merge::MergeStrategy;
use ratatui::widgets::Block;

pub trait LayoutExt {
    fn collapse_if(self, condition: bool) -> Self;
}

impl LayoutExt for Layout {
    fn collapse_if(self, condition: bool) -> Self {
        if condition {
            self.spacing(-1)
        } else {
            self
        }
    }
}

pub trait BlockExt<'a> {
    fn merge_if(self, condition: bool) -> Block<'a>;
}

impl<'a> BlockExt<'a> for Block<'a> {
    fn merge_if(self, condition: bool) -> Block<'a> {
        if condition {
            self.merge_borders(MergeStrategy::Exact)
        } else {
            self
        }
    }
}
