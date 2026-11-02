# Напівпрозора сіра заливка деталей у `draw_pieces`

## Контекст
Зараз `draw_pieces` малює лише контур деталі. Деталі мають заливатися напівпрозорим сірим, щоб їхню площу було видно на полотні, а сітку під ними — крізь заливку. "Фон" тут означає внутрішню область кожної деталі, а не все полотно.

## Зміни
Файл: `src/editor/canvas.rs`.

1. Додати `use earcutr::earcut;`. Крейт уже є залежністю `desktop` і використовується в `src/preview/mod.rs` (`build_piece_mesh`).
2. Додати допоміжну функцію:

   ```rust
   fn fill_polygon(painter: &egui::Painter, screen: &[egui::Pos2], color: egui::Color32) {
       let flat: Vec<f64> = screen
           .iter()
           .flat_map(|position| [f64::from(position.x), f64::from(position.y)])
           .collect();
       let Ok(triangles) = earcut(&flat, &[], 2) else {
           return;
       };
       let mut mesh = egui::Mesh::default();
       for position in screen {
           mesh.colored_vertex(*position, color);
       }
       for triangle in triangles.chunks_exact(3) {
           mesh.add_triangle(triangle[0] as u32, triangle[1] as u32, triangle[2] as u32);
       }
       painter.add(mesh);
   }
   ```

   Тріангуляція потрібна, бо деталі можуть бути неопуклими, а `egui::Shape::convex_polygon` коректно заливає лише опуклі багатокутники.
3. У `draw_pieces` оголосити поруч із `text_color`:
   `let fill_color = egui::Color32::from_rgba_unmultiplied(128, 128, 128, 80);`
   і викликати `fill_polygon(painter, &screen, fill_color);` безпосередньо перед `closed_line`, щоб контур, назва й вершини лишалися поверх заливки.

Заливка однакова для вибраних і невибраних деталей. Вибір і далі показує контур. Логіка вибору та hit testing не змінюються.

`docs/project-index.md`: оновити кількість рядків `canvas.rs` і діапазон розділу Painting, додавши `fill_polygon`.

## Перевірка
- `cargo fmt`, `cargo check --bin leathercraft`, `cargo clippy`.
- `cargo run`: намалювати прямокутник, коло й неопуклу деталь (наприклад, L-подібну) інструментом багатокутника. Заливка має точно повторювати форму, зокрема у виїмці; сітку видно крізь неї; під час перетягування вершини заливка йде за нею.
