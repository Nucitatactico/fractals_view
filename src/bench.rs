//! Benchmark secuencial contra paralelo, sin interfaz gráfica.
//!
//!   cargo run --release -- --bench
//!   cargo run --release -- --bench --reps 10 --csv resultados.csv
//!   cargo run --release -- --bench --escalado      # además: tiempo según número de hilos
//!   cargo run --release -- --bench --rapido        # profundidades y tamaños reducidos
//!
//! Imprime tablas en Markdown (se pueden pegar tal cual en el reporte) y, con `--csv`,
//! guarda todos los datos (min, mediana, media y desviación) en un archivo.
 
use crate::fractales::{
    arbol_pitagoras::ArbolPitagoras,
    copo_koch::CopoKoch,
    dragon::Dragon,
    mandelbrot::{Mandelbrot, ModoMandelbrot},
    romanesco::Romanesco,
};
use crate::hilos;
use std::hint::black_box;
use std::io::Write;
use std::time::Instant;
 
type Func = Box<dyn Fn() -> usize + Send + Sync>;
 
struct Caso {
    fractal: &'static str,
    detalle: String,
    /// El caso más grande de cada fractal se usa en el estudio de escalado.
    escalar: bool,
    secuencial: Func,
    paralelo: Func,
}
 
impl Caso {
    fn nuevo<F, G>(fractal: &'static str, detalle: String, escalar: bool, seq: F, par: G) -> Self
    where
        F: Fn() -> usize + Send + Sync + 'static,
        G: Fn() -> usize + Send + Sync + 'static,
    {
        Self {
            fractal,
            detalle,
            escalar,
            secuencial: Box::new(seq),
            paralelo: Box::new(par),
        }
    }
}
 
struct Stats {
    min: f64,
    mediana: f64,
    media: f64,
    desv: f64,
}
 
fn stats(tiempos: &[f64]) -> Stats {
    let mut o = tiempos.to_vec();
    o.sort_by(|a, b| a.total_cmp(b));
    let n = o.len();
    let media = o.iter().sum::<f64>() / n as f64;
    let varianza = o.iter().map(|x| (x - media).powi(2)).sum::<f64>() / n as f64;
    let mediana = if n % 2 == 1 {
        o[n / 2]
    } else {
        (o[n / 2 - 1] + o[n / 2]) / 2.0
    };
    Stats { min: o[0], mediana, media, desv: varianza.sqrt() }
}
 
/// Una ejecución de calentamiento (caché, asignador, hilos de rayon) y `reps` medidas.
fn cronometrar(f: &Func, reps: usize) -> (Stats, usize) {
    let _ = black_box(f());
    let mut tiempos = Vec::with_capacity(reps);
    let mut elementos = 0;
    for _ in 0..reps {
        let t0 = Instant::now();
        elementos = black_box(f());
        tiempos.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    (stats(&tiempos), elementos)
}
 
fn casos(rapido: bool) -> Vec<Caso> {
    let mut v = Vec::new();
 
    let koch: &[u32] = if rapido { &[8, 10] } else { &[8, 10, 12] };
    for (i, &p) in koch.iter().enumerate() {
        v.push(Caso::nuevo(
            "Copo de Koch",
            format!("profundidad {p}"),
            i == koch.len() - 1,
            move || CopoKoch::medir(p, false),
            move || CopoKoch::medir(p, true),
        ));
    }
 
    let pit: &[u32] = if rapido { &[16, 20] } else { &[16, 20, 22] };
    for (i, &p) in pit.iter().enumerate() {
        v.push(Caso::nuevo(
            "Árbol de Pitágoras",
            format!("profundidad {p}"),
            i == pit.len() - 1,
            move || ArbolPitagoras::medir(p, false),
            move || ArbolPitagoras::medir(p, true),
        ));
    }
 
    let dra: &[u32] = if rapido { &[14, 16] } else { &[14, 16, 18] };
    for (i, &p) in dra.iter().enumerate() {
        v.push(Caso::nuevo(
            "Curva del Dragón",
            format!("profundidad {p}"),
            i == dra.len() - 1,
            move || Dragon::medir(p, false),
            move || Dragon::medir(p, true),
        ));
    }
 
    let rom: &[u32] = if rapido { &[2] } else { &[2, 3] };
    for (i, &p) in rom.iter().enumerate() {
        v.push(Caso::nuevo(
            "Brócoli Romanesco",
            format!("profundidad {p}"),
            i == rom.len() - 1,
            move || Romanesco::medir(p, false),
            move || Romanesco::medir(p, true),
        ));
    }
 
    let tamanos: &[(usize, usize, u32)] = if rapido {
        &[(800, 500, 500)]
    } else {
        &[(800, 500, 500), (1600, 1000, 1000)]
    };
    for (i, &(w, h, it)) in tamanos.iter().enumerate() {
        let ultimo = i == tamanos.len() - 1;
        for (nombre, modo) in [
            ("Mandelbrot (bandas)", ModoMandelbrot::Bandas),
            ("Mandelbrot (intercalado)", ModoMandelbrot::Intercalado),
        ] {
            v.push(Caso::nuevo(
                nombre,
                format!("{w}x{h} / {it} iter"),
                ultimo,
                move || Mandelbrot::medir(w, h, it, ModoMandelbrot::Secuencial),
                move || Mandelbrot::medir(w, h, it, modo),
            ));
        }
    }
    v
}
 
fn valor(args: &[String], clave: &str) -> Option<String> {
    args.iter()
        .position(|a| a == clave)
        .and_then(|i| args.get(i + 1))
        .cloned()
}
 
/// 1, 2, 4, 8... hasta el número de núcleos lógicos (incluido).
fn lista_hilos(max: usize) -> Vec<usize> {
    let mut v = Vec::new();
    let mut n = 1;
    while n < max {
        v.push(n);
        n *= 2;
    }
    v.push(max);
    v
}
 
struct FilaCsv {
    fractal: &'static str,
    detalle: String,
    modo: String,
    hilos: usize,
    elementos: usize,
    s: Stats,
    speedup: f64,
}
 
pub fn ejecutar(args: &[String]) {
    let reps: usize = valor(args, "--reps")
        .and_then(|v| v.parse().ok())
        .unwrap_or(5)
        .max(1);
    let csv = valor(args, "--csv");
    let escalado = args.iter().any(|a| a == "--escalado");
    let rapido = args.iter().any(|a| a == "--rapido");
 
    let nucleos = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    let hilos_pool = rayon::current_num_threads();
 
    println!("# Benchmark: secuencial contra paralelo\n");
    println!("- Núcleos lógicos: {nucleos} | hilos del pool de rayon: {hilos_pool} | trabajadores lógicos (reparto): {}", hilos::trabajadores());
    println!("- Repeticiones: {reps} (más 1 de calentamiento) | tiempos en milisegundos, se reporta la mediana");
    if cfg!(debug_assertions) {
        println!("- AVISO: compilado en modo debug; usa `cargo run --release -- --bench` para tiempos válidos");
    }
 
    let casos = casos(rapido);
    let mut filas: Vec<FilaCsv> = Vec::new();
    let mut medianas_seq: Vec<f64> = Vec::new();
 
    println!("\n## Comparación\n");
    println!("| Fractal | Detalle | Elementos | Secuencial (ms) | Paralelo (ms) | Speedup | Eficiencia |");
    println!("|---|---|---:|---:|---:|---:|---:|");
 
    for c in &casos {
        let (s_seq, n_seq) = cronometrar(&c.secuencial, reps);
        let (s_par, n_par) = cronometrar(&c.paralelo, reps);
        let speedup = s_seq.mediana / s_par.mediana;
        let eficiencia = speedup / hilos_pool as f64;
 
        println!(
            "| {} | {} | {} | {:.2} | {:.2} | {:.2}x | {:.0}% |",
            c.fractal,
            c.detalle,
            n_par,
            s_seq.mediana,
            s_par.mediana,
            speedup,
            eficiencia * 100.0
        );
 
        medianas_seq.push(s_seq.mediana);
        filas.push(FilaCsv {
            fractal: c.fractal,
            detalle: c.detalle.clone(),
            modo: "secuencial".into(),
            hilos: 1,
            elementos: n_seq,
            s: s_seq,
            speedup: 1.0,
        });
        filas.push(FilaCsv {
            fractal: c.fractal,
            detalle: c.detalle.clone(),
            modo: "paralelo".into(),
            hilos: hilos_pool,
            elementos: n_par,
            s: s_par,
            speedup,
        });
    }
 
    if escalado {
        println!("\n## Escalado con el número de hilos\n");
        let lista = lista_hilos(nucleos);
        for (i, c) in casos.iter().enumerate().filter(|(_, c)| c.escalar) {
            let seq = medianas_seq[i];
            println!("**{} ({})**, secuencial: {:.2} ms\n", c.fractal, c.detalle, seq);
            println!("| Hilos | Paralelo (ms) | Speedup vs secuencial | Speedup vs 1 hilo | Eficiencia |");
            println!("|---:|---:|---:|---:|---:|");
 
            let mut con_un_hilo: Option<f64> = None;
            for &n in &lista {
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(n)
                    .build()
                    .expect("no se pudo crear el pool de hilos");
                let (s, elementos) = pool.install(|| cronometrar(&c.paralelo, reps));
 
                let base = *con_un_hilo.get_or_insert(s.mediana);
                let speedup = seq / s.mediana;
                println!(
                    "| {} | {:.2} | {:.2}x | {:.2}x | {:.0}% |",
                    n,
                    s.mediana,
                    speedup,
                    base / s.mediana,
                    base / s.mediana / n as f64 * 100.0
                );
 
                filas.push(FilaCsv {
                    fractal: c.fractal,
                    detalle: c.detalle.clone(),
                    modo: "paralelo_escalado".into(),
                    hilos: n,
                    elementos,
                    s,
                    speedup,
                });
            }
            println!();
        }
    }
 
    if let Some(ruta) = csv {
        match std::fs::File::create(&ruta) {
            Ok(mut f) => {
                let _ = writeln!(
                    f,
                    "fractal,detalle,modo,hilos,reps,elementos,min_ms,mediana_ms,media_ms,desv_ms,speedup"
                );
                for r in &filas {
                    let _ = writeln!(
                        f,
                        "\"{}\",\"{}\",{},{},{},{},{:.3},{:.3},{:.3},{:.3},{:.3}",
                        r.fractal, r.detalle, r.modo, r.hilos, reps, r.elementos,
                        r.s.min, r.s.mediana, r.s.media, r.s.desv, r.speedup
                    );
                }
                println!("\nDatos guardados en {ruta}");
            }
            Err(e) => eprintln!("No se pudo crear {ruta}: {e}"),
        }
    }
}
 
