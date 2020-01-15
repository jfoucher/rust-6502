use cursive::views::*;
use cursive::view::*;


pub fn layout() -> cursive::views::LinearLayout {
    let mut mem_view = LinearLayout::vertical();
    let mut cnt = 0;

    for i in 0..32 {
        let mut h = LinearLayout::horizontal();
        h.add_child(
            TextView::new("").with_id(format!("addr-{}", cnt).as_str())
        );
        h.add_child(DummyView.fixed_width(2));
        for j in 0..16 {
            h.add_child(
                TextView::new("").with_id(format!("mem-{}", cnt).as_str())
            );
            if (j + 1) % 4 == 0 && j < 15 {
                h.add_child(DummyView.fixed_width(2));
            } else if j < 15 {
                h.add_child(DummyView.fixed_width(1));
            }
            
            cnt += 1;
        }

        mem_view.add_child(h);
    }

    LinearLayout::horizontal()
        .child(
            LinearLayout::vertical()
            .child(
                Dialog::around(
                    mem_view
                ).title("Memory")
            )
            .child(
                Dialog::around(
                    TextView::new("OUTPUT").with_id("output")
                ).title("Output")
            )
        )
        .child(
            LinearLayout::vertical()
            .child(Dialog::around(
                LinearLayout::horizontal()
                .child(
                    LinearLayout::vertical()
                    .child(
                        TextView::new("Flags")
                    )
                    .child(
                        TextView::new("").with_id("flags").fixed_width(10)
                    )
                )
                .child(
                    DummyView.fixed_width(3)
                )
                .child(
                    LinearLayout::vertical()
                    .child(
                        TextView::new("PC")
                    )
                    .child(
                        TextView::new("").with_id("pc").fixed_width(15)
                    )
                )
                .child(
                    DummyView.fixed_width(3)
                )
                .child(
                    LinearLayout::vertical()
                    .child(
                        TextView::new("Acc")
                    )
                    .child(
                        TextView::new("").with_id("acc").fixed_width(5)
                    )
                )
                .child(
                    DummyView.fixed_width(3)
                )
                .child(
                    LinearLayout::vertical()
                    .child(
                        TextView::new("Rx")
                    )
                    .child(
                        TextView::new("").with_id("rx").fixed_width(5)
                    )
                )
                .child(
                    DummyView.fixed_width(3)
                )
                .child(
                    LinearLayout::vertical()
                    .child(
                        TextView::new("Ry")
                    )
                    .child(
                        TextView::new("").with_id("ry").fixed_width(5)
                    )
                )
                .child(
                    DummyView.fixed_width(3)
                )
                .child(
                    LinearLayout::vertical()
                    .child(
                        TextView::new("SP")
                    )
                    .child(
                        TextView::new("").with_id("sp").fixed_width(5)
                    )
                )
                .child(
                    DummyView.fixed_width(3)
                )
                .child(
                    LinearLayout::vertical()
                    .child(
                        TextView::new("Clock")
                    )
                    .child(
                        TextView::new("").with_id("clock").fixed_width(10)
                    )
                )
                
            ).title("Processor info").fixed_width(80))
            .child(Dialog::around(
                TextView::new("PROC INFO").with_id("info")
            ).title("Debug info").fixed_width(80).scrollable())
            .child(Dialog::around(
                TextView::new("T").with_id("test")
            ).title("Current test").fixed_width(80).scrollable())
            .child(Dialog::around(
                TextView::new("T").with_id("stack")
            ).title("Stack").fixed_width(80).scrollable())
            
        )
            
}


