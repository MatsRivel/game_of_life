use rand::{self, thread_rng, Rng};
use std::{thread, time};
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator, IntoParallelIterator};
fn nested_get(arr_in:[[i32;GRID_SIZE];GRID_SIZE], x_in:usize, y_in: usize) -> Option<i32>{
    match arr_in.get(x_in){
        Some(second_arr) => match second_arr.get(y_in){
            Some(v) => Some(v+0), // Add 0 to not have it as a reference any more. #TODO: Improve!
            None => None,
        },
        None => None,
    }
}

fn make_multiple_sub_arr(base_grid_in:[[i32;GRID_SIZE];GRID_SIZE],sub_number:usize) -> Option<Vec<[[i32;Y_SUB_DIM];X_SUB_DIM]>>{
    // NOTE: Could use global variables here, but as I am trying to get rid of them i will keep these.
    if sub_number*X_SUB_DIM > base_grid_in.len(){
        return None;
    }
    let mut sub_vec = vec![[[0;Y_SUB_DIM];X_SUB_DIM];N_CHUNKS];
    for n in 0..N_CHUNKS{
        let col_per_row = GRID_SIZE/Y_SUB_DIM;
        let x_start = X_SUB_DIM * (n/col_per_row);
        let y_start = Y_SUB_DIM * (n%col_per_row);
        populate_sub_arr(base_grid_in, &mut sub_vec[n], x_start, y_start);
    }
    return Some(sub_vec);
}

fn populate_sub_arr(base_grid_in:[[i32;GRID_SIZE];GRID_SIZE], sub_in: &mut [[i32;Y_SUB_DIM];X_SUB_DIM], x_start:usize, y_start:usize){
    let x_end = x_start + X_SUB_DIM;
    let y_end = y_start + Y_SUB_DIM;
    for x_idx in x_start..x_end{
        for y_idx in y_start..y_end{
            sub_in[x_idx-x_start][y_idx-y_start] = match nested_get(base_grid_in, x_idx, y_idx) {
                Some(v) => v,
                None => 0,
            }
        }
    }

}

fn get_n_neighbours(neighbourhood:[[i32;3];3])->i32{
    let mut output = 0;
    for i in 0..3{
        for j in 0..3{
            if (i,j) != (1,1){
                output += neighbourhood[i][j];
            };
        }
    }
    return output;
}

fn life_rules(life_status:i32, neighbourhood:[[i32;3];3]) -> i32{
    //Rules:
    // Living with 2 or 3 neighbours get to live.
    // Dead with 3 neighbours comes alive.
    // All other cells remain or become dead.
    let n_neighbours = get_n_neighbours(neighbourhood);
    println!("nbrs: {}", n_neighbours);
    return (life_status*n_neighbours == 2 || life_status*n_neighbours == 3 || (1+life_status)*n_neighbours == 3) as i32
}

fn make_padded_neighbours(grid:[[i32;Y_SUB_DIM];X_SUB_DIM],x_idx:usize, y_idx:usize) -> [[i32;3];3]{
    //TODO: This is where the problem lies.
    let mut output = [[0i32;3];3];
    let (x_dim,y_dim) = (grid.len()-1,grid.len()-1);
    let (x_start, y_start, x_end, y_end) = match(x_idx, y_idx){
        (0usize,0usize)             => (0usize,  0usize,  x_idx+1, y_idx+1),
        (0usize,y_dim)       => (0usize,  y_idx-1, x_idx+1, y_dim  ),
        (x_dim,0usize)       => (x_idx-1, 0usize,  x_dim,   y_idx+1),
        (x_dim,y_dim) => (x_idx-1, y_idx-1, x_dim,   y_dim  ),
        (_,_)                       => (x_idx-1, y_idx-1, x_idx+1, y_idx+1)
    };
    for x in x_start..(x_start+3){
        if x > x_end{//Break if hits the edge
            break;
        }
        for y in y_start..(y_start+3){
            if y > y_end{// Break if hits the edge
                break;
            }
            output[x][y] = grid[x][y];

        }
    }
    return output;

}
// TODO: Currently using global constant. Find a better option.
const GRID_BASE: usize = 9;
const GRID_PADDING: usize = 0;
const GRID_SIZE: usize = GRID_BASE + GRID_PADDING;
const X_DIM:usize = GRID_SIZE;
const Y_DIM:usize = X_DIM;
const X_SUB_DIM:usize = 3;
const Y_SUB_DIM:usize = X_SUB_DIM;
const N_CHUNKS:usize = (X_DIM/X_SUB_DIM) * (Y_DIM/Y_SUB_DIM); 
const N_LIVING_CELLS: i32 = 0;//((GRID_BASE*GRID_BASE)/4usize )as i32;

fn main() {
    /*
    Implement "Conways Game of Life":
    1) Create a grid of cells represented as a 2D array / vector
    2) Initialize the gird with a pattern of dead/living cells
    3) Split grid into smaller sub-grids so that they can be processed in parallel by threads.
    4) Create a thread pool using "rayon" (Previously used std; Use that instead?)
    5) For each thread, compute the next sub-grid. Update the main grid.
    6) Syncronize the results. Update the main grid.
    7) Repeat 4-7 until complete.
    */
    const HALF_GRID_AREA: i32 = (GRID_SIZE*GRID_SIZE/2usize) as i32;
    match N_LIVING_CELLS {
        0i32..=HALF_GRID_AREA => {}, // Continue as normal.
        _ => panic!("Too many living cells for current implementation of seeding the grid.") // TODO: Handle instead of panic.
    }
    // 1) Create a grid of cells represented as a 2D array / vector
    let mut base_grid = [[1;GRID_SIZE];GRID_SIZE];
    
    // 2) Initialize the gird with a pattern of dead/living cells
    let mut rng = thread_rng();
    for _ in 0..N_LIVING_CELLS{ // Seed the grid with n living cells:
        loop {  // Loops to prevent overlap, but this can cause issues if N_LIVING_CELLS ~= GRID_SIZE^2. Therefor there is a guard at the top preventing this.
                // Should be improved to prevent this limitation.
            let row = rng.gen_range(GRID_PADDING..GRID_BASE);
            let column = rng.gen_range(GRID_PADDING..GRID_BASE);
            match base_grid[row as usize][column as usize] {
                0 => {
                    base_grid[row as usize][column as usize] = 1;
                    break;
                }
                _ => {}
            }
        }
    }
    println!("N_CHUNKS: {}", N_CHUNKS);
    

    for _element in base_grid{
        println!("{:?}",_element);
    }
    println!();
    // 3) Split grid into smaller sub-grids so that they can be processed in parallel by threads.
    //    - m, n are the dimentions of the grid.
    //    - k, l are the dimentions of the sub-grid.
    //      NOTE: Currently, overflow will not be considered, but will be lost.
    //      Make sure GRID_SIZE%X_SUB_DIM == 0
    for _ in 0..2{ // This loop is for testing only.
        thread::sleep(time::Duration::from_millis(100));
        let mut sub_grids = Vec::<[[i32;Y_SUB_DIM];X_SUB_DIM]>::with_capacity(N_CHUNKS);
        let mut n = 0;
        loop{
            let sub = match make_multiple_sub_arr(base_grid, n){
                Some(v) => v,
                None => break
            };
            for s in sub{
                sub_grids.push(s);
                n+=1;
            }
        }
        // 3.5) Make the thing work without threading, then add threading.
        //  -   Update subgrids based on rules.
        for n in 0..N_CHUNKS{
            for x in 0..X_SUB_DIM{
                for y in 0..Y_SUB_DIM{
                    println!("XXXXXXXXXXXXXXXXXX");
                    for i in 0..3{
                        for j in 0..3{
                            print!("{} ",sub_grids[n][i][j]);
                        }
                        println!();
                    }
                    println!("XXXXXXXXXXXXXXXXXX");
                    println!("__________________");

                    let padded_neighbour_grid = make_padded_neighbours(sub_grids[n], x, y);
                    for i in 0..3{
                        for j in 0..3{
                            print!("{} ",padded_neighbour_grid[i][j]);
                        }
                        println!();
                    }
                    let life_state = padded_neighbour_grid[1][1]; // Center of the 3x3 grid.
                    let temp = life_rules(life_state, padded_neighbour_grid);
                    println!("New life: {} | Old life: {}",temp, life_state);
                    sub_grids[n][x][y] = temp;
                    println!("__________________");

                }
            }
            println!();
        }
        println!();
        //  -   Update main-grid based on sub-grids.
        for n in 0..N_CHUNKS{
            let grid = sub_grids[n];
            for x in 0..X_SUB_DIM{
                for y in 0..Y_SUB_DIM{
                    let i = x + n%X_SUB_DIM;
                    let j = y + n%Y_SUB_DIM;
                    //println!("({}, {}) | ({}, {}) | {}",x,y,i,j,n);
                    base_grid[i][j] = grid[x][y];
                }
            }
        }
        for _element in base_grid{
            println!("{:?}",_element);
        }
        println!();
    }

    /*
    // 4) Create a thread pool using "rayon" (Previously used std; Use that instead?)
    const N_THREADS:usize = 8usize;
    let thread_pool = match rayon::ThreadPoolBuilder::new().num_threads(N_THREADS).build(){
        Ok(successfull_pool) => successfull_pool,
        Err(error_message) =>panic!("{}",error_message), // TODO: Handle error instead.
    };
    */


}
