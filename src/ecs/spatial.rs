#[derive(Clone, Debug)]
pub struct SpatialGrid {
    pub cell_size: f64,
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<Vec<hecs::Entity>>>,
}

impl SpatialGrid {
    pub fn new(canvas_width: f64, canvas_height: f64, cell_size: f64) -> Self {
        let width = ((canvas_width / cell_size).ceil() as usize).max(1);
        let height = ((canvas_height / cell_size).ceil() as usize).max(1);
        let cells = vec![vec![Vec::new(); height]; width];

        Self {
            cell_size,
            width,
            height,
            cells,
        }
    }

    pub fn clear(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                cell.clear();
            }
        }
    }

    pub fn get_cell(&self, x: f64, y: f64) -> (usize, usize) {
        let grid_x = (x / self.cell_size).floor().max(0.0) as usize;
        let grid_y = (y / self.cell_size).floor().max(0.0) as usize;
        (grid_x.min(self.width - 1), grid_y.min(self.height - 1))
    }

    pub fn add_entity(&mut self, entity: hecs::Entity, x: f64, y: f64) {
        let (grid_x, grid_y) = self.get_cell(x, y);
        self.cells[grid_x][grid_y].push(entity);
    }

    pub fn get_nearby_entities(&self, x: f64, y: f64, radius: f64) -> Vec<hecs::Entity> {
        let mut entities = Vec::new();
        let (center_x, center_y) = self.get_cell(x, y);
        let cell_radius = (radius / self.cell_size).ceil() as i32;

        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                let check_x = center_x as i32 + dx;
                let check_y = center_y as i32 + dy;

                if check_x >= 0
                    && check_x < self.width as i32
                    && check_y >= 0
                    && check_y < self.height as i32
                {
                    entities.extend(
                        self.cells[check_x as usize][check_y as usize]
                            .iter()
                            .copied(),
                    );
                }
            }
        }

        entities
    }
}
