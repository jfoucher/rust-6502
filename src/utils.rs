use cursive::views::*;
use cursive::view::*;


pub fn layout() -> cursive::views::LinearLayout {

    LinearLayout::horizontal()
        .child(Dialog::around(
            TextView::new("TEST MEM").with_id("memory")
        ).title("Memory"))
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
                        TextView::new("").with_id("flags")
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
                        TextView::new("").with_id("pc")
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
                        TextView::new("").with_id("acc")
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
                        TextView::new("").with_id("rx")
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
                        TextView::new("").with_id("ry")
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
                        TextView::new("").with_id("sp")
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
                        TextView::new("").with_id("clock")
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