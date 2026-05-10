# bin2img

Converts compiled binaries (EXE, DLL, ELF, etc.) into fixed-width 2D byte
matrices for malware visualization in MATLAB.

Implements the Nataraj et al. (2011) binary visualization technique:
> Nataraj, L., et al. "Malware images: visualization and automatic classification."
> Proceedings of the 8th international symposium on visualization for cyber security. 2011.

## Build

```bash
cargo build --release
# Binary: ./target/release/bin2img
```

## Usage

```bash
# Single file → writes calc.bin (raw u8) next to the binary
./bin2img /usr/bin/ls

# Custom output location
./bin2img /usr/bin/ls -o ./output/ls_matrix

# CSV format (slower, human-readable, larger)
./bin2img /usr/bin/ls --csv

# Batch — process every file in a directory
./bin2img ./malware_samples/ --batch
```

## Output

**Raw mode (default, recommended):**
- `<stem>.bin` — flat row-major u8 array, always 256 columns, N rows

Load in MATLAB:
```matlab
fid = fopen('output.bin', 'rb');
img = fread(fid, [256 Inf], 'uint8')';
fclose(fid);
img = mat2gray(img);
```

**CSV mode:**
- `<stem>.csv` — same data, comma-separated integers
```matlab
img = mat2gray(csvread('output.csv'));
```

## Why 256 columns?

256 is the standard in binary visualization research (Nataraj 2011). Each row
maps to exactly 256 bytes of the file's address space, giving the x-axis a
consistent semantic meaning across all binaries. Dynamic square dimensions look
prettier but break cross-file comparison.

## MATLAB pipeline (Phase 2+)

See `load_binary_image.m` — a ready-to-run script that covers:
1. Loading the .bin file
2. mat2gray normalization (Preprocessing Technique 1)
3. Pseudo-colormap enhancement (Preprocessing Technique 2)
4. Entropy filtering — highlights packed/encrypted regions
5. Otsu's thresholding — binary segmentation of suspicious sections
6. MSE / PSNR metric comparison between benign and malware samples

## Safe sample sources

- **MalwareBazaar** (bazaar.abuse.ch) — free, no signup, downloadable samples
- **VirusShare** — larger corpus, requires account
- Keep samples as raw bytes only, never execute them.
  On Zorin/Linux you're not the execution target anyway, but still — VM or
  a dedicated `~/malware/` directory with `chmod 000` on the actual EXE files.
