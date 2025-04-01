use eframe::egui;
use std::ops::Add;

// 定义椭圆曲线上的点
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: Option<i64>,
    y: Option<i64>,
    a: i64,
    b: i64,
    p: i64,
}

impl Point {
    fn new(x: i64, y: i64, a: i64, b: i64, p: i64) -> Self {
        Point {
            x: Some(x % p),
            y: Some(y % p),
            a,
            b,
            p,
        }
    }

    fn infinity(a: i64, b: i64, p: i64) -> Self {
        Point {
            x: None,
            y: None,
            a,
            b,
            p,
        }
    }

    fn mod_inverse(a: i64, p: i64) -> i64 {
        fn extended_gcd(a: i64, b: i64) -> (i64, i64, i64) {
            if a == 0 {
                (b, 0, 1)
            } else {
                let (gcd, x1, y1) = extended_gcd(b % a, a);
                let x = y1 - (b / a) * x1;
                let y = x1;
                (gcd, x, y)
            }
        }
        let (_, x, _) = extended_gcd(a, p);
        (x % p + p) % p
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        if self.x.is_none() {
            return other;
        }
        if other.x.is_none() {
            return self;
        }

        let x1 = self.x.unwrap();
        let y1 = self.y.unwrap();
        let x2 = other.x.unwrap();
        let y2 = other.y.unwrap();
        let p = self.p;

        if x1 == x2 && (y1 + y2) % p == 0 {
            return Point::infinity(self.a, self.b, p);
        }

        let lambda: i64;
        if x1 == x2 && y1 == y2 {
            lambda = ((3 * x1 * x1 + self.a) * Point::mod_inverse(2 * y1, p)) % p;
        } else {
            lambda = ((y2 - y1) * Point::mod_inverse(x2 - x1, p)) % p;
        }

        let x3 = ((lambda * lambda - x1 - x2) % p + p) % p; // 规范化
        let y3 = ((lambda * (x1 - x3) - y1) % p + p) % p; // 规范化
        Point::new(x3, y3, self.a, self.b, p)
    }
}

// 找到所有满足曲线的点
fn find_points(a: i64, b: i64, p: i64) -> Vec<Point> {
    let mut points = Vec::new();
    for x in 0..p {
        let rhs = (x * x * x + a * x + b) % p;
        for y in 0..p {
            if (y * y) % p == rhs {
                points.push(Point::new(x, y, a, b, p));
            }
        }
    }
    points
}

// GUI 应用程序
struct EllipticCurveApp {
    points: Vec<Point>,
    steps: Vec<Point>,
    p: i64,
}

impl EllipticCurveApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let p = 599; // 大素数
        let a = 1;
        let b = 1;
        let points = find_points(a, b, p);

        let mut steps = vec![];
        let mut point = Point::new(5, 1, 1, 1, p);
        for _k in 0..=20 {
            point = point.clone() + point;
            steps.push(point.clone());
        }
        EllipticCurveApp { points, p, steps }
    }
}

impl eframe::App for EllipticCurveApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Elliptic Curve Points (y^2 = x^3 + x + 1 mod 599)");

            // 绘制点的区域
            let (width, height) = (599.0, 599.0);
            let painter = ui.painter().clone();
            let rect = ui.allocate_space(egui::Vec2::new(width, height)).1;

            // 绘制网格背景
            let step = width / self.p as f32;
            for i in 0..=self.p {
                let x = rect.min.x + i as f32 * step;
                let y = rect.min.y + i as f32 * step;
                painter.line_segment(
                    [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                    egui::Stroke::new(1.0, egui::Color32::LIGHT_GRAY),
                );
                painter.line_segment(
                    [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
                    egui::Stroke::new(1.0, egui::Color32::LIGHT_GRAY),
                );
            }

            // 绘制点
            for point in &self.points {
                if let (Some(x), Some(y)) = (point.x, point.y) {
                    assert!(x >= 0 && x < self.p);
                    assert!(y >= 0 && y < self.p);
                    let px = rect.min.x + (x as f32 * step);
                    let py = rect.max.y - (y as f32 * step); // y 轴翻转，0 在底部
                    painter.circle_filled(egui::pos2(px, py), 2.0, egui::Color32::RED);
                }
            }

            for (i, point) in self.steps.iter().enumerate() {
                if let (Some(x), Some(y)) = (point.x, point.y) {
                    assert!(x >= 0 && x < self.p);
                    assert!(y >= 0 && y < self.p);
                    let px = rect.min.x + (x as f32 * step);
                    let py = rect.max.y - (y as f32 * step); // y 轴翻转，0 在底部
                    let color = if i == self.steps.len() - 1 {
                        egui::Color32::YELLOW
                    } else {
                        egui::Color32::BLUE
                    };
                    painter.circle_filled(egui::pos2(px, py), 4.0, color);
                }
            }

            for i in 0..=self.steps.len() {
                if i < self.steps.len() - 1 {
                    let p1 = &self.steps[i];
                    let p2 = &self.steps[i + 1];
                    if let (Some(x1), Some(y1)) = (p1.x, p1.y) {
                        if let (Some(x2), Some(y2)) = (p2.x, p2.y) {
                            let px1 = rect.min.x + (x1 as f32 * step);
                            let py1 = rect.max.y - (y1 as f32 * step);
                            let px2 = rect.min.x + (x2 as f32 * step);
                            let py2 = rect.max.y - (y2 as f32 * step);
                            painter.line_segment(
                                [egui::pos2(px1, py1), egui::pos2(px2, py2)],
                                egui::Stroke::new(2.0, egui::Color32::BLUE),
                            );
                        }
                    }
                }
            }

            ui.label(format!("20 * p = {:?}", self.steps.last().unwrap()));
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Elliptic Curve Visualizer",
        options,
        Box::new(|cc| Ok(Box::new(EllipticCurveApp::new(cc)))),
    )
}
