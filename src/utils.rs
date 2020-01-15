use cursive::views::*;
use cursive::view::*;


pub fn layout() -> cursive::views::LinearLayout {
    let mut mem_view = LinearLayout::vertical();
    let mut cnt = 0;

    for i in 0..32 {
        let mut h = LinearLayout::horizontal();
        h.add_child(
            TextView::new("aa").with_id(format!("addr-{}", cnt).as_str())
        );
        mem_view.add_child(h);
        cnt += 1;
    }
    cnt = 0;

    let mut st = LinearLayout::vertical();
    for i in 0..16 {
        st.add_child(
            TextView::new("aa").with_id(format!("stack-{}", cnt).as_str())
        );

        cnt += 1;
    }

    LinearLayout::horizontal()
        .child(
            LinearLayout::vertical()
            .child(
                Dialog::around(
                    mem_view
                ).title("Memory")
            )
            // .child(
            //     Dialog::around(
            //         TextView::new("OUTPUT").with_id("output")
            //     ).title("Output")
            // )
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
                .child(
                    DummyView.fixed_width(2)
                )
                .child(
                    LinearLayout::vertical()
                    .child(
                        TextView::new("Speed")
                    )
                    .child(
                        TextView::new("").with_id("speed").fixed_width(8)
                    )
                )
                
            ).title("Processor info"))
            .child(Dialog::around(
                TextView::new("PROC INFO").with_id("info")
            ).title("Debug info").fixed_width(84).fixed_height(20).scrollable())
            .child(
                LinearLayout::horizontal().child(
                    Dialog::around(
                        st
                    ).title("Stack").fixed_width(64).fixed_height(18)
                )
                .child(
                    Dialog::around(
                        TextView::new("0").with_id("test")
                    ).title("Current test").fixed_width(20)
                )
                
            )
            
        )
            
}


