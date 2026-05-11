# MalwareRizz

> **Binary Visualization for Malware Detection using Digital Image Processing**

MalwareRizz is a cross-platform malware analysis pipeline that treats compiled
binaries as images. Every executable — whether a Windows PE, Linux ELF, or
shared library — is a raw stream of bytes. Reshape that stream into a 256-column
2D matrix, apply spatial Digital Image Processing techniques, and the internal
structure of the binary becomes visible: code sections show repeating patterns,
null-padded regions appear as uniform dark bands, and malicious payloads that
have been packed or encrypted to evade antivirus detection manifest as regions
of extreme visual chaos.

This technique was first formalized by Nataraj et al. (2011). MalwareRizz
implements and extends it through a 4-phase pipeline: a Rust CLI for fast
binary-to-matrix conversion, MATLAB preprocessing and visualization, automated
entropy-based detection with a 7-feature scoring engine, and quantitative
evaluation via MSE and PSNR.

**Academic context:** CPE 415 — Digital Image Processing Lab,
COMSATS University Islamabad, Lahore Campus.

---

## Table of Contents

1. [Repository Structure](#repository-structure)
2. [Prerequisites](#prerequisites)
3. [Phase 1 — bin2img (Rust CLI)](#phase-1--bin2img-rust-cli)
4. [Phase 2 — Visualization Pipeline (MATLAB)](#phase-2--visualization-pipeline-matlab)
5. [Phase 3 — Automated Detection (MATLAB)](#phase-3--automated-detection-matlab)
6. [Understanding the Results](#understanding-the-results)
7. [Safe Sample Sources](#safe-sample-sources)
8. [Roadmap](#roadmap)
9. [Team](#team)
10. [References](#references)

---

## Repository Structure

```
bin2img/
├── src/
│   └── main.rs                  # Rust CLI source
├── Cargo.toml                   # Rust project manifest
├── Cargo.lock
│
├── Malware_viz_pipeline.m       # Phase 2: side-by-side visualization (report figures)
├── malware_autodetect.m         # Phase 3: automated detection engine + verdict
│
├── ls_matrix.bin                # Sample benign matrix (ls binary, gitignore in prod)
├── malware_matrix.bin           # Sample malware matrix (gitignore in prod)
│
└── README.md
```

> `load_binary_image.m` is an old Phase 2 draft — **it is obsolete**.
> Use `Malware_viz_pipeline.m` and `malware_autodetect.m` instead.

---

## Prerequisites

### Rust (for bin2img)
```bash
# Install rustup if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustc --version   # verify: rustc 1.7x.x
```

### MATLAB
Any of the following work:
- **MATLAB Online** — [matlab.mathworks.com](https://matlab.mathworks.com) (free with university account)
- MATLAB R2019b or later (local install)

Required toolbox: **Image Processing Toolbox**
(provides `mat2gray`, `entropyfilt`, `graythresh`, `imbinarize`, `psnr`)

---

## Phase 1 — bin2img (Rust CLI)

Reads any compiled binary as raw bytes, chunks them into 256-byte rows,
zero-pads the last row, and writes the result as a flat `.bin` file ready
for MATLAB.

### Build

```bash
cd bin2img/
cargo build --release
# Output: ./target/release/bin2img
```

### Usage

```bash
# Single file — output next to input
./target/release/bin2img /usr/bin/ls

# Custom output path (no extension needed)
./target/release/bin2img /usr/bin/ls -o ~/Desktop/ls_matrix

# Batch mode — convert every file in a directory
./target/release/bin2img ./malware_samples/ --batch

# CSV format instead of raw binary (slower, larger, human-readable)
./target/release/bin2img /usr/bin/ls --csv
```

### Output

**Raw binary mode (default, recommended):**

Writes `<stem>.bin` — a flat row-major array of `u8` bytes, always 256 columns,
N rows (where N = ceil(filesize / 256)).

The tool also prints the MATLAB snippet to load it:

```
[raw] ls
size   : 142312 bytes → image 256×556 px
output : /home/faizy/Desktop/ls_matrix.bin
done in: 550µs

% MATLAB load:
% fid = fopen('ls_matrix.bin', 'rb');
% img = fread(fid, [256 Inf], 'uint8')';
% fclose(fid);
% img = mat2gray(img);
```

**CSV mode:**

Writes `<stem>.csv` — same data, comma-separated integers.
Useful for inspection but ~4× larger and slower to load.

```matlab
img = mat2gray(csvread('output.csv'));
```

### Why 256 columns?

256 is the standard width in binary visualization research (Nataraj 2011).
Each row maps to exactly one 256-byte block of the file's address space,
so the x-axis has a consistent semantic meaning across all binaries regardless
of file size. Dynamic square dimensions look nicer but destroy cross-file
comparability.

---

## Phase 2 — Visualization Pipeline (MATLAB)

**Script:** `Malware_viz_pipeline.m`

This script produces the six labeled figures used in the report. It takes a
benign reference and an unknown sample and runs them through the full DIP
pipeline side by side.

### Setup

1. Generate `.bin` files for both samples using bin2img
2. Upload both `.bin` files to MATLAB Online (Files panel → Upload)
3. Upload `Malware_viz_pipeline.m`
4. Edit the two filenames at the top of the script

```matlab
BENIGN_FILE  = 'ls_matrix.bin';
MALWARE_FILE = 'malware_matrix.bin';
```

5. Run with **F5**

### What it produces

| Figure | DIP Technique | What you see |
|--------|--------------|--------------|
| 1. Grayscale — Benign vs Malware | Normalisation (`mat2gray`) | Raw byte texture; code sections show repeating patterns |
| 2. Pseudo-color jet — Benign vs Malware | False-color enhancement | Blue = null padding, yellow/green = code, red = dense data |
| 3. Pseudo-color parula — Benign vs Malware | Perceptually uniform colormap | Same data, publication-quality coloring |
| 4. Entropy filter — Benign vs Malware | `entropyfilt` (9×9 neighbourhood) | Hot = high entropy = packed/encrypted = suspicious |
| 5. Otsu segmentation — Benign vs Malware | `graythresh` + `imbinarize` | White = high-entropy (suspicious), black = structured code |
| 6. Entropy histogram — Benign vs Malware | Distribution comparison | Benign: wide spread; Malware: sharp peak at high entropy |

### Console output (sample)

```
FINAL RESULTS
════════════════════════════════════
  Benign  mean entropy : 4.0573
  Malware mean entropy : 4.4509
  Entropy delta        : 0.3936
  MSE                  : 0.076857
  PSNR                 : 11.1432 dB
════════════════════════════════════
```

---

## Phase 3 — Automated Detection (MATLAB)

**Script:** `malware_autodetect.m`

The main detection engine. Takes a known-benign reference and an unknown
binary, computes 7 independent image-feature signals, weights each signal,
and outputs a justified verdict: **MALWARE**, **SUSPICIOUS**, or **LIKELY CLEAN**.

### Setup

Same upload process as Phase 2. Edit the config block at the top:

```matlab
BENIGN_REF   = 'ls_matrix.bin';      % known-good baseline
UNKNOWN_FILE = 'malware_matrix.bin'; % file under test
SAMPLE_LABEL = 'unknown_sample';     % label shown in report
```

Run with **F5**.

### The 7 detection signals

| # | Signal | Weight | Fires when |
|---|--------|--------|------------|
| F1 | Absolute entropy mean | 2.0 | Mean local entropy > 6.80 |
| F2 | Entropy delta vs benign | 2.5 | Unknown entropy > benign by > 0.40 |
| F3 | High-entropy pixel % | 2.0 | Otsu mask covers > 55% of pixels |
| F4 | MSE (entropy maps) | 1.0 | MSE > 0.04 |
| F5 | PSNR (entropy maps) | 1.0 | PSNR < 22 dB |
| F6 | Byte std deviation ratio | 1.0 | std(unknown) / std(benign) > 1.30 |
| F7 | Chi-squared histogram divergence | 0.5 | χ² > 0.01 |

**Total possible weight: 10.0**

### Verdict thresholds

| Confidence | Verdict |
|-----------|---------|
| ≥ 70% | 🔴 MALWARE |
| 40–69% | 🟠 SUSPICIOUS |
| < 40% | 🟢 LIKELY CLEAN |

### What it produces

**Console — detection report:**
```
╔══════════════════════════════════════════════════════════╗
║         MALWARE DETECTION REPORT                        ║
╠══════════════════════════════════════════════════════════╣
║  File           : unknown_sample                        ║
║  Verdict        : SUSPICIOUS                            ║
║  Evidence score : 4.50 / 10.00  (confidence: 45.0%)    ║
╠══════════════════════════════════════════════════════════╣
║  EVIDENCE FOR MALWARE                                   ║
╠══════════════════════════════════════════════════════════╣
║  [+2.0] 57.0% of pixels are high-entropy (thr 55.0%)   ║
║  [+1.0] Entropy-map MSE=0.07686 > 0.0400               ║
║  [+1.0] PSNR=11.14 dB < 22.0 dB                        ║
║  [+0.5] Byte histogram chi2=7.15839 > 0.01              ║
╠══════════════════════════════════════════════════════════╣
║  CLEAN INDICATORS                                       ║
╠══════════════════════════════════════════════════════════╣
║  [ OK ] Entropy mean normal: 4.4509 <= 6.80             ║
║  [ OK ] Entropy close to benign (delta=0.3936)          ║
║  [ OK ] Byte std ratio normal: 0.875                    ║
╚══════════════════════════════════════════════════════════╝
```

**Figure 1 — Malware Detection Dashboard (3×4 grid):**
- Row 1: Grayscale images + jet colormap (benign vs unknown)
- Row 2: Entropy maps + Otsu segmentation masks
- Row 3: Byte distribution histograms + verdict panel

**Figure 2 — Feature Radar Chart:**
All 7 signals normalized to [0, 1] and plotted as a spider chart.
Spikes toward the edge = that signal fired strongly.

---

## Understanding the Results

### What the entropy map tells you

| Region colour (hot colormap) | Meaning |
|-----------------------------|---------|
| Deep red / bright white | Very high entropy — packed, encrypted, or compressed |
| Yellow / orange | Moderate entropy — typical of compiled code (.text section) |
| Dark / black | Low entropy — null padding, zero-filled memory, headers |

### Why SUSPICIOUS ≠ definitely malware

The scoring engine compares an unknown file against a **single** benign
reference. A legitimately compressed binary (e.g. a UPX-packed open-source
tool) will score SUSPICIOUS because it looks statistically similar to packed
malware. The verdict reflects structural anomaly, not a signature match.
A heavily packed trojan or ransomware typically scores MALWARE (≥ 70%) due to
extreme entropy uniformity across the entire binary.

### The Otsu threshold caveat

Otsu's method computes a threshold independently for each image, maximising
inter-class variance within that image. This means the high-entropy percentage
figures are not directly comparable across samples — a benign file can show
81% "high entropy" if its own Otsu threshold is low. Use **mean entropy** and
**MSE/PSNR** as the primary cross-file comparison metrics; the Otsu mask is
best read visually.

---

## Safe Sample Sources

| Source | URL | Notes |
|--------|-----|-------|
| MalwareBazaar | [bazaar.abuse.ch](https://bazaar.abuse.ch) | Free, no signup, ELF + PE samples, ZIP password: `infected` |
| VirusShare | [virusshare.com](https://virusshare.com) | Larger corpus, requires free account |
| System binaries (benign) | `/usr/bin/*` on any Linux install | Perfect benign baselines — use `ls`, `python3`, `bash` |

**Safety rules:**
- Never execute downloaded samples
- Work with raw bytes only — bin2img reads the file, it never runs it
- On Linux (Zorin OS) you are not the execution target for Windows PE malware anyway
- For extra caution: `chmod 000 malware.exe` after downloading

---

## Roadmap

### Planned

- [ ] **MATLAB App Designer GUI** — drag-and-drop `.bin` file, single Run button,
  all dashboard figures render inside the app window, verdict displayed as a
  large coloured label. Eliminates the need to manually upload files and run
  scripts. Fully supported on MATLAB Online.

- [ ] **Rust CLI UX improvement** — interactive mode where the tool prompts for
  an input file instead of requiring a full path argument. Reduces friction
  for non-technical users during the demo.

- [ ] **Multi-sample comparison** — extend `malware_autodetect.m` to accept a
  directory of unknowns and produce a results table: one row per binary,
  showing all 7 feature values and the final verdict. Useful for evaluating
  detection accuracy across a labelled corpus.

### Stretch goals

- [ ] **CNN classifier** — train a lightweight convolutional network directly
  on the binary visualization images (following Nataraj 2011 Section 4).
  Would replace the rule-based scoring engine with a learned classifier.

- [ ] **Configurable reference baseline** — instead of a single benign reference
  file, use an average entropy profile computed from N benign binaries. Makes
  F1/F2/F4/F5 signals more robust against variation in individual clean files.

---

## Team

| Name | Reg. No. | Contribution |
|------|----------|-------------|
| Muhammad Faizan Sharjeel | FA22-BCE-086 | Rust CLI, system architecture, pipeline design |
| Junaid Zaheer | FA22-BCE-045 | MATLAB preprocessing & enhancement |
| Muqadas Imtiaz | FA22-BCE-030 | Filtering, segmentation, evaluation, report |

---

## References

Nataraj, L., Karthikeyan, S., Jacob, G., & Manjunath, B. S. (2011).
*Malware images: Visualization and automatic classification.*
Proceedings of the 8th International Symposium on Visualization for Cyber
Security (VizSec '11). ACM. https://doi.org/10.1145/2016904.2016908

MalwareBazaar — Abuse.ch malware sample repository.
https://bazaar.abuse.ch

The Rust Programming Language — Official documentation.
https://doc.rust-lang.org

MATLAB Image Processing Toolbox — MathWorks documentation.
https://www.mathworks.com/help/images/
