
pub struct PagelistItem {
    pub text: String,
    pub current: bool,
    pub page: i32
}

pub fn get_pagelist(total: i32, page_size: i32, current: i32) -> Vec<PagelistItem>
{
    let mut pagelist = Vec::new();

    for i in (0..total).step_by(page_size as usize) {
        let thispage = i / page_size;
        pagelist.push(PagelistItem {
            page: thispage + 1,
            text: format!("{}", thispage + 1),
            current: thispage == current
        });
    }

    pagelist
}
