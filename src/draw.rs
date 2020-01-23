use unicode_width::UnicodeWidthChar;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_dash_line() {
        assert_eq!(format_dash_line(5), "-----");
        assert_eq!(format_dash_line(1), "-")
    }

    #[test]
    fn test_format_with_dash_wrap() {
        assert_eq!(format_with_dash_wrap("hehe", 10, 1, ""), "-- hehe --");
        assert_eq!(format_with_dash_wrap("heh", 10, 1, ""), "-- heh ---");
        assert_eq!(format_with_dash_wrap("hehe", 11, 1, ""), "-- hehe ---");
        assert_eq!(format_with_dash_wrap("hehe", 10, 0, ""), "---hehe---");
        assert_eq!(format_with_dash_wrap("heh", 12, 2, ""), "--  heh  ---");
        assert_eq!(format_with_dash_wrap("index", 13, 1, "|+"), "|+- index -+|");
        // TODO MORE TEST
    }
}

//// # example
//// ```rust
//// let mut s = String::from("你好啊!");
//// filter_wid_char!(s);
//// assert_eq!(s, "***!");
#[macro_export]
macro_rules! filter_wid_char {
    ($s:ident) => {
        $s = $s
            .chars()
            .map(|c| if c.len_utf8() == 1 { c } else { '*' })
            .collect();
    };
}

//// # example:
//// format_dash_line(5);
//// // output: "-----"
#[inline]
#[allow(dead_code)] // useful
pub fn format_dash_line(len: usize) -> String {
    format!("{:-^1$}", "", len)
}

//// # example:
//// format_space_line(5);
//// // output: "     "
#[inline]
pub fn format_space_line(len: usize) -> String {
    format!("{:^1$}", "", len)
}

//// if len equal to 0, output len is 0.
#[inline]
pub fn format_with_space_wrap(s: &str, max_len: usize) -> String {
    let mut remain = max_len;
    let mut extra_width = 0;
    let s: String = s
        .chars()
        .fold(Vec::new(), |mut v, c| {
            if let Some(wid) = UnicodeWidthChar::width(c) {
                if remain > wid {
                    v.push(c);
                    extra_width += wid - 1;
                    remain -= wid;
                }
            };
            v
        })
        .into_iter()
        .collect();
    let mut res = String::new();
    if max_len > 0 {
        res += format!("{0: ^1$}", s, max_len - extra_width).as_str();
    }
    res
}

macro_rules! change_line {
    ($s: ident) => {
        $s += "\n";
    };
}

//// # example:
//// ```
//// format_with_dash_wrap("hehe", 10, 1, "");
//// // output: "-- hehe --"
//// format_with_dash_wrap("hhh", 10, 1, "");
//// // output: "-- heh ---"
//// format_with_dash_wrap("hehe", 11, 1, "");
//// // output: "-- hehe ---"
//// ```
//// # Remark
//// not like `format_with_space_wrap`, you should always expect fn return a
//// dash-wrapped String, as you must make sure the max_len is friendly.
pub fn format_with_dash_wrap(s: &str, max_len: usize, padd: usize, margin: &str) -> String {
    let s = if s.len() > max_len - (2 * margin.len()) - (padd * 2) {
        &s[..max_len - (2 * margin.len()) - (padd * 2)]
    } else {
        s
    };
    let remain = max_len - s.len();
    let len_l = (remain - (2 * padd) - (2 * margin.len())) / 2;
    let len_r = if remain % 2 > 0 { len_l + 1 } else { len_l };
    format!(
        "{5}{1:-^2$}{1:^4$}{0}{1:^4$}{1:-^3$}{6}",
        s,
        "",
        len_l,
        len_r,
        padd,
        margin,
        margin.chars().rev().collect::<String>()
    )
}

#[macro_export]
macro_rules! format_rows {
    ($win:expr, $data:expr, &[$($field:tt);*]) => {
        {
        let mut vs = Vec::new();
        for d in $data.iter() {
            let mut vd = vec![];
            $ (
                vd.push(d.$field.to_string());
            ) *
            if let Ok(row) = $win.build_row(&vd[..]) {
                vs.push(row);
            }
        };
        vs
        }
    }
}

#[macro_export]
macro_rules! print_rows {
    ($win:expr, $data:expr, &[$($field:tt);*]) => {
        {
        for d in $data.iter() {
            let mut vd = vec![];
            $ (
                vd.push(d.$field.to_string());
            ) *
            if let Ok(row) = $win.build_row(&vd[..]) {
                print!("{}", row)
            }
        };
        }
    }
}

#[macro_export]
macro_rules! print_title {
    ($win: expr) => {
        print!("{}", $win.build_title().unwrap());
    };
}

#[macro_export]
macro_rules! print_foot {
    ($win: expr) => {
        print!("{}", $win.build_foot());
    };
}

#[macro_export]
macro_rules! print_div {
    ($win: expr, $s: expr, $color:tt ) => {
        print!("{}", $win.build_div($s).unwrap().$color());
    };
}

pub mod sbui {

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_symbol_window() {
            let mut sw = SymbolWindow::new();

            // resize
            let test_width = 50;
            sw.resize(test_width);
            assert_eq!(sw.get_width(), test_width);

            let tags = ["name", "age", "gender"];
            // insert
            sw.add_tag(&tags);
            match sw.get_tag(tags.len() - 1) {
                None => panic!("should be gender"),
                Some(s) => assert_eq!(s, "gender"),
            };

            // not exist label's weight is 0
            assert_eq!(sw.get_weight_ratio("sdfsdfsdfsdf") == 0.0, true);

            // each label's weight is 0.33...
            assert_eq!(sw.get_weight_ratio("name") < 0.34, true);
            assert_eq!(sw.get_weight_ratio("name") > 0.32, true);

            // double first label's weight
            sw.change_weight("name", |n| 2.0 * n);

            // should be 0.5
            assert_eq!(sw.get_weight_ratio("name") > 0.49, true);
            assert_eq!(sw.get_weight_ratio("name") < 0.51, true);

            match sw.get_weight("name") {
                Some(w) => assert_eq!(w, DEFAULT_INIT_WEIGHT * 2),
                None => panic!("wrong weight"),
            }
            sw.refresh();
            match sw.get_col_width() {
                Err(_) => panic!("count width failed"),
                Ok(vw) => {
                    let div = (tags.len() + 1) as i32;
                    let total_width = vw.iter().fold(div, |sum, &wid| sum + wid);
                    assert_eq!(total_width, test_width);
                }
            }
            // build title
            sw.resize_title(0);
            let title = match sw.build_title() {
                Err(msg) => panic!(msg),
                Ok(t) => t,
            };
            let expect_title = "+------------------------------------------------+\n".to_string()
                + "|         name          |    age     |  gender   |\n"
                + "+------------------------------------------------+\n";
            assert_eq!(title, expect_title);

            struct TestTemp {
                name: String,
                age: i16,
                gender: i16,
            }

            let data = vec![
                TestTemp {
                    name: String::from("data1"),
                    age: 1,
                    gender: 4,
                },
                TestTemp {
                    name: String::from("data2"),
                    age: 2,
                    gender: 5,
                },
                TestTemp {
                    name: String::from("data3"),
                    age: 3,
                    gender: 6,
                },
            ];
            let rs = format_rows!(sw, data, &[name; age; gender]);
            let expect_rs = vec![
                String::from("|         data1         |     1      |     4     |\n"),
                String::from("|         data2         |     2      |     5     |\n"),
                String::from("|         data3         |     3      |     6     |\n"),
            ];
            assert_eq!(rs, expect_rs);
        }
    }

    use super::format_space_line;
    use super::format_with_dash_wrap;
    use super::format_with_space_wrap;

    const DEFAULT_INIT_WEIGHT: i32 = 100;
    const DEFAULT_DIV_SPACE_PADDING: i32 = 1;
    const DEFAULT_LABEL_HEIGHT: i32 = 1;
    const DEFAULT_LR_BORDER_SINGLE: &str = "|";
    const DEFAULT_DIV_SPACE_PADDING_OUTER: i32 = 1;
    const DEFAULT_LR_BORDER_CORNER: &str = "+";
    const DEFAULT_WINDOW_WIDTH:i32 = 80;
    const DEFAULT_WINDOW_HEIGHT:i32 = 20;

    struct SymbolWindowLabel {
        tag: String,
        weight: i32,
        wid: i32,
    }

    impl SymbolWindowLabel{
        fn new(tag: &str) -> Self {
            SymbolWindowLabel {
                tag:String::from(tag),
                weight:DEFAULT_INIT_WEIGHT,
                wid:0,
            }
        }
    }

    pub struct SymbolWindow {
        col_label: Vec<SymbolWindowLabel>,
        width: i32,
        label_v_padd: i32,
        clean: bool,
    }

    impl SymbolWindow {
        pub fn new() -> Self {
            SymbolWindow {
                width:DEFAULT_WINDOW_WIDTH,
                col_label: Vec::new(),
                clean: true,
                label_v_padd: DEFAULT_LABEL_HEIGHT,
            }
        }

        fn get_total_weight(&self) -> i32 {
            self.col_label.iter().fold(0, |w, l| w + l.weight)
        }

        #[allow(dead_code)] // useful
        pub fn get_width(&self) -> i32 {
            self.width
        }

        #[allow(dead_code)] // useful
        pub fn resize(&mut self, width: i32) {
            self.width = width;
            self.clean = false;
        }

        #[allow(dead_code)] // useful
        pub fn resize_title(&mut self, v_padd: i32) {
            self.label_v_padd = v_padd;
        }

        pub fn add_tag(&mut self, names: &[&str]) {
            names.iter().for_each(|&name| {
                self.col_label.push(SymbolWindowLabel::new(name));
            });
            self.clean = false;
        }

        pub fn get_tag(&self, idx: usize) -> Option<&str> {
            if idx < self.col_label.len() {
                Some(self.col_label[idx].tag.as_str())
            } else {
                None
            }
        }

        #[allow(dead_code)] // useful
        fn get_weight(&self, name: &str) -> Option<i32> {
            match self.col_label.iter().find(|l| l.tag == name) {
                Some(l) => Some(l.weight),
                None => None,
            }
        }

        #[allow(dead_code)] // useful
        pub fn get_weight_ratio(&self, name: &str) -> f64 {
            match self.get_weight(name) {
                None => 0.0,
                Some(s) => s as f64 / self.get_total_weight() as f64,
            }
        }

        fn count_width(&mut self) -> Result<(), &'static str> {
            let label_count = self.col_label.len();
            let total_f = self.get_total_weight() as f64;
            let valid_space = self.width - 1 - (label_count as i32);
            let mut free_space = valid_space;
            if free_space <= 0 {
                return Err("no more space");
            }

            self.col_label.iter_mut().for_each(|l| {
                let w = ((l.weight as f64 / total_f) * (valid_space as f64)).round() as i32;
                let w = if free_space - w >= 0 {
                    free_space -= w;
                    w
                } else {
                    let last_part = free_space;
                    free_space = 0;
                    last_part
                };
                l.wid = w;
            });
            if free_space > 0 {
                if let Some(last) = self.col_label.last_mut() {
                    last.wid += free_space;
                }
            }
            self.clean = true;
            Ok(())
        }

        //// almost every `mut` call, should following with `refresh` to update the col width
        pub fn refresh(&mut self) {
            if !self.clean {
                if let Err(_) = self.count_width() {
                    //refresh failed
                }
            }
        }

        pub fn get_col_width(&self) -> Result<Vec<i32>, ()> {
            if !self.clean {
                // if let Ok(vw) = self.count_width() {
                //     self.col_width = vw;
                // }
                return Err(());
            }
            let wids:Vec<i32> = self.col_label.iter().map(|l|l.wid).collect();
            Ok(wids)
        }

        //// # example
        //// ```
        ////|                          |
        ////```
        #[allow(dead_code)] // useful
        fn build_empty_line(&self) -> Result<String, ()> {
            let mut s = String::new();

            s += DEFAULT_LR_BORDER_SINGLE;
            let wid = if self.width > 2 * DEFAULT_LR_BORDER_SINGLE.len() as i32 {
                self.width as i32 - 2 * DEFAULT_LR_BORDER_SINGLE.len() as i32
            } else {
                return Err(());
            };
            s += format_space_line(wid as usize).as_str();
            s += DEFAULT_LR_BORDER_SINGLE;
            change_line!(s);
            Ok(s)
        }

        fn build_title_space_line(&self) -> Result<String, ()> {
            let mut s = String::new();

            for _ in 0..self.label_v_padd {
                if let Ok(row) = self.build_row::<&str>(&[]) {
                    s += row.as_str();
                }
            }
            Ok(s)
        }

        pub fn build_title(&self) -> Result<String, &'static str> {
            let mut s = String::new();
            s += format_with_dash_wrap("", self.width as usize, 0, DEFAULT_LR_BORDER_CORNER)
                .as_str();
            change_line!(s);
            if let Ok(line) = self.build_title_space_line() {
                s += line.as_str();
            }
            let vw = match self.get_col_width() {
                Ok(vw) => vw,
                Err(_) => return Err("no space"),
            };
            s += DEFAULT_LR_BORDER_SINGLE;
            for (i, &wid) in vw.iter().enumerate() {
                match self.get_tag(i) {
                    None => return Err("data mismatch"),
                    Some(tag) => {
                        s += (format_with_space_wrap(tag, wid as usize) + DEFAULT_LR_BORDER_SINGLE)
                            .as_str()
                    }
                }
            }
            change_line!(s);
            if let Ok(line) = self.build_title_space_line() {
                s += line.as_str();
            }
            s += format_with_dash_wrap("", self.width as usize, 0, DEFAULT_LR_BORDER_CORNER)
                .as_str();
            change_line!(s);
            Ok(s)
        }

        pub fn build_div<T: AsRef<str>>(&self, name: T) -> Result<String, ()> {
            let border_style = String::from(DEFAULT_LR_BORDER_SINGLE)
                + format_space_line(DEFAULT_DIV_SPACE_PADDING_OUTER as usize).as_str();
            let mut r = format_with_dash_wrap(
                name.as_ref(),
                self.width as usize,
                DEFAULT_DIV_SPACE_PADDING as usize,
                border_style.as_str(),
            );
            change_line!(r);
            Ok(r)
        }

        pub fn build_foot(&self) -> String {
            let mut r = format_with_dash_wrap("", self.width as usize, 0, DEFAULT_LR_BORDER_CORNER);
            change_line!(r);
            r
        }

        pub fn build_row<T: AsRef<str>>(&self, data: &[T]) -> Result<String, &'static str> {
            let vw = match self.get_col_width() {
                Ok(vw) => vw,
                Err(_) => return Err("no space"),
            };
            let mut s = String::from(DEFAULT_LR_BORDER_SINGLE);
            for (i, &wid) in vw.iter().enumerate() {
                match data.get(i) {
                    None => {
                        s += (format_space_line(wid as usize) + DEFAULT_LR_BORDER_SINGLE).as_str()
                    }
                    Some(d) => {
                        s += (format_with_space_wrap(d.as_ref(), wid as usize)
                            + DEFAULT_LR_BORDER_SINGLE)
                            .as_str()
                    }
                }
            }
            change_line!(s);
            Ok(s)
        }

        pub fn change_weight<F>(&mut self, name: &str, mul: F)
        where
            F: Fn(f64) -> f64,
        {
            let total_weight = self.get_total_weight() as f64;
            for label in self.col_label.iter_mut() {
                if label.tag != name {
                    continue;
                }
                let rat = label.weight as f64 / total_weight;
                let expect_rat = mul(rat);
                if expect_rat > 0.0 && expect_rat < 1.0 {
                    let new_weight = (expect_rat * total_weight) as i32;
                    let diff = new_weight - label.weight;
                    if label.weight + diff > 0 {
                        label.weight += diff;
                        self.clean = false;
                    }
                }
            }
        }
    }
}
