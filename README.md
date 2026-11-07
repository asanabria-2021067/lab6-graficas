# Lab 6: Shaders — Solmira, Tessara y Orvane

Tres cuerpos celestes para el futuro sistema solar de fantasía, todos dibujados con la **misma esfera** (`assets/sphere.obj`, la del Canvas, sin modificar vértices) y el **software renderer en CPU**. No hay texturas ni imágenes externas: todo el color sale de los fragment shaders, con la posición del fragmento, su normal, la luz, el tiempo y ruido OpenSimplex.

| Solmira · estrella | Tessara · planeta rocoso | Orvane · gigante gaseoso |
|---|---|---|
| ![](docs/anim_estrella.gif) | ![](docs/anim_rocoso.gif) | ![](docs/anim_gaseoso.gif) |

Extras con la misma esfera:

| Albiceleste · balón | Lumen · orbe de cristal |
|---|---|
| ![](docs/anim_balon.gif) | ![](docs/anim_orbe.gif) |

---

## Compilar y ejecutar

Requisitos: Rust (edición 2024) y lo que necesita raylib para compilar: CMake y un compilador de C. En Linux además `libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev`.

```bash
cargo run --release
```

> Usen siempre `--release`: en modo debug el ruido es mucho más lento.

### Controles

| Tecla | Acción |
|---|---|
| **1 / 2 / 3** | Estrella / planeta rocoso / gigante gaseoso (cada uno con su propio encuadre de cámara y luz) |
| **4 / 5** | Extras: balón albiceleste / orbe de cristal |
| Espacio | Pausar el tiempo |
| R | Activar / desactivar el giro de la esfera |
| L | La luz da vueltas sola · **Q / E** mover la luz a mano |
| Flechas | Orbitar la cámara · **W / S** o rueda: acercar / alejar |
| H | Resolución completa o media (más FPS) |
| K | Guardar captura PNG en `screenshots/` |
| **Estrella** | **O / P** cantidad de manchas · **U / I** velocidad del plasma |
| **Rocoso** | **B** relieve · **, / .** fuerza del relieve · **C** nubes · **X** sombras de nubes · **[ / ]** cobertura de nubes · **Z** brillo especular · **N** ciudades · **- / =** nivel del mar |
| **Gaseoso** | **9 / 0** fuerza de la tormenta · **T / Y** turbulencia de las bandas |
| **Balón** | **J / ;** cantidad de rayas |
| **Orbe** | **F / G** refracción del vidrio · **V / M** relieve del núcleo |

En la ventana se ven los valores actuales de cada parámetro, así se puede demostrar qué cambia al moverlos.

### Modo sin ventana (capturas y GIFs)

Todas las imágenes de este README salen del mismo programa:

```bash
cargo run --release -- capture          # todo: docs/*.png y docs/frames/*
cargo run --release -- capture sombras  # solo las capturas cuyo nombre contiene "sombras"
bash tools/make_gifs.sh                 # convierte los cuadros en GIFs (necesita ffmpeg)
```

Los escenarios de cada captura (tiempo, cámara, luz y parámetros) están en `src/capture.rs`.

---
