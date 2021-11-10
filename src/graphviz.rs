use std::io::Write;

pub fn render(hp: &[u32], wasm_mem: &[u32]) -> std::io::Result<tempfile::TempPath> {
    assert_eq!(hp.len(), wasm_mem.len());
    let n_points = hp.len();

    let (mut hp_csv_file, hp_csv_path) = tempfile::Builder::new().tempfile()?.into_parts();

    writeln!(hp_csv_file, "hp")?;
    for hp in hp {
        writeln!(hp_csv_file, "{}", hp)?;
    }
    hp_csv_file.flush()?;
    drop(hp_csv_file);

    let (mut wasm_mem_csv_file, wasm_mem_csv_path) =
        tempfile::Builder::new().tempfile()?.into_parts();
    writeln!(wasm_mem_csv_file, "Wasm mem")?;
    for wasm_mem in wasm_mem {
        writeln!(wasm_mem_csv_file, "{}", wasm_mem)?;
    }
    wasm_mem_csv_file.flush()?;
    drop(wasm_mem_csv_file);

    let gnuplot = GNUPLOT_TEMPLATE
        .replace("$XRANGE", &n_points.to_string())
        .replace("$HP_CSV_PATH", hp_csv_path.to_str().unwrap())
        .replace("$WASM_MEM_CSV_PATH", wasm_mem_csv_path.to_str().unwrap());

    let (mut gnuplot_file, gnuplot_file_path) = tempfile::Builder::new().tempfile()?.into_parts();
    gnuplot_file.write_all(gnuplot.as_bytes())?;
    drop(gnuplot_file);

    let (png_file, png_file_path) = tempfile::Builder::new().tempfile()?.into_parts();

    let gnuplot_out = std::process::Command::new("gnuplot")
        .arg(gnuplot_file_path.to_str().unwrap())
        .stdout(std::process::Stdio::from(png_file))
        .output()?;

    if !gnuplot_out.status.success() {
        println!(
            "gnuplot failed, stderr={}",
            String::from_utf8_lossy(&gnuplot_out.stderr)
        );
    }

    Ok(png_file_path)
}

static GNUPLOT_TEMPLATE: &str = r###"
set terminal png notransparent rounded giant font "JetBrains Mono" 24 \
  size 1200,960 

set xtics nomirror
set ytics nomirror

set style line 80 lt 0 lc rgb "#808080"

set border 3 back ls 80 

set style line 81 lt 0 lc rgb "#808080" lw 0.5

set grid xtics
set grid ytics
set grid mxtics
set grid mytics

set grid back ls 81

set style line 1 lt 1 lc rgb "#A00000" lw 2 pt 7 ps 1.5
set style line 2 lt 1 lc rgb "#00A000" lw 2 pt 11 ps 1.5

set datafile separator ','

set xlabel "call"
set ylabel "bytes"

set xrange [0:$XRANGE]

plot "$HP_CSV_PATH" using 0:1 with linespoints title "HP (canister allocs)", \
     "$WASM_MEM_CSV_PATH" using 0:1 with linespoints title "Wasm memory (runtime allocs)"
"###;
