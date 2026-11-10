//! Feasibility witness only: a four-cycle, two parts, fixed C-METIS seed.
//! This is not a replacement for BISECT's engine or its acceptance matrix.

/// Encode cut and exact assignments in a small return value, or -1 on failure.
#[no_mangle]
pub extern "C" fn partition_probe() -> i32 {
    let xadj = [0, 2, 4, 6, 8];
    let adjncy = [1, 3, 0, 2, 1, 3, 0, 2];
    let mut assignments = [0; 4];
    let graph = match metis::Graph::new(1, 2, &xadj, &adjncy) {
        Ok(graph) => graph,
        Err(_) => return -1,
    };
    match graph
        .set_option(metis::option::Seed(7))
        .part_kway(&mut assignments)
    {
        Ok(cut) => assignments
            .iter()
            .enumerate()
            .fold(cut << 8, |value, (index, part)| value | (part << index)),
        Err(_) => -1,
    }
}

/// Exercise the native signal/longjmp cleanup path on a disconnected graph.
#[no_mangle]
pub extern "C" fn disconnected_probe() -> i32 {
    let graph = match metis::Graph::new(1, 2, &[0, 0, 0], &[]) {
        Ok(graph) => graph,
        Err(_) => return -2,
    };
    match graph
        .set_option(metis::option::Contig(true))
        .part_kway(&mut [0; 2])
    {
        Ok(_) => 1,
        Err(metis::Error::Input) => -2,
        Err(metis::Error::Memory) => -3,
        Err(metis::Error::Other) => -4,
    }
}

/// Invalid refinement count reaches native option validation (METIS_ERROR_INPUT).
#[no_mangle]
pub extern "C" fn invalid_option_probe() -> i32 {
    let graph = metis::Graph::new(1, 2, &[0, 2, 4, 6, 8], &[1, 3, 0, 2, 1, 3, 0, 2]).unwrap();
    match graph
        .set_option(metis::option::NIter(0))
        .part_kway(&mut [0; 4])
    {
        Ok(_) => 1,
        Err(metis::Error::Input) => -2,
        Err(metis::Error::Memory) => -3,
        Err(metis::Error::Other) => -4,
    }
}
