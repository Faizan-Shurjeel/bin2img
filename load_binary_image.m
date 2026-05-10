% load_binary_image.m
% Phase 2 starter — load a bin2img output and apply preprocessing.
% Run this after bin2img has exported your .bin files.
%
% USAGE: edit the FILENAME variable at the top, then Run Section by section.

%% ── 0. Config ───────────────────────────────────────────────────────────────
FILENAME = 'sample.bin';   % ← change to your exported .bin file
ROW_WIDTH = 256;           % must match bin2img (always 256)

%% ── 1. Load raw bytes ───────────────────────────────────────────────────────
fid = fopen(FILENAME, 'rb');
if fid == -1
    error('Cannot open file: %s', FILENAME);
end
raw = fread(fid, [ROW_WIDTH Inf], 'uint8')';   % [rows × 256]
fclose(fid);

fprintf('Loaded: %d rows × %d cols (%d bytes total)\n', ...
    size(raw,1), size(raw,2), numel(raw));

%% ── 2. Preprocessing — normalisation ────────────────────────────────────────
% Technique 1: mat2gray maps [0,255] → [0.0, 1.0]
img_gray = mat2gray(raw);

figure('Name', '1. Grayscale (raw bytes)');
imshow(img_gray);
title('Raw byte visualization — grayscale');
colorbar;

%% ── 3. Enhancement — pseudo-colormap ────────────────────────────────────────
% Technique 2: false-colour map makes structural sections visible.
% 'jet'    → classic blue-green-red; .text sections appear blue/green,
%            zero padding appears deep blue, packed regions appear red/white.
% 'parula' → perceptually uniform; better for publication figures.
figure('Name', '2. Pseudo-color (jet)');
imshow(img_gray);
colormap(jet);
title('Pseudo-color enhancement — jet colormap');
colorbar;

figure('Name', '3. Pseudo-color (parula)');
imshow(img_gray);
colormap(parula);
title('Pseudo-color enhancement — parula colormap');
colorbar;

%% ── 4. Filtering — entropy map ──────────────────────────────────────────────
% High entropy = packed/encrypted/compressed regions → potential malice.
% entropyfilt uses a 9×9 neighbourhood by default.
fprintf('Computing entropy filter (may take a few seconds)...\n');
img_entropy = entropyfilt(img_gray);

figure('Name', '4. Entropy filter');
imshow(mat2gray(img_entropy));
colormap(hot);
title('Entropy map — hot regions = high entropy (suspicious)');
colorbar;

%% ── 5. Segmentation — Otsu thresholding ─────────────────────────────────────
% Automatically finds threshold that maximises inter-class variance.
thresh      = graythresh(mat2gray(img_entropy));
img_binary  = imbinarize(mat2gray(img_entropy), thresh);

figure('Name', '5. Segmentation (Otsu)');
imshow(img_binary);
title(sprintf('Otsu segmentation — threshold = %.4f', thresh));

fprintf('Otsu threshold: %.4f\n', thresh);
fprintf('High-entropy pixels: %d / %d (%.1f%%)\n', ...
    sum(img_binary(:)), numel(img_binary), ...
    100 * sum(img_binary(:)) / numel(img_binary));

%% ── 6. Evaluation metrics ───────────────────────────────────────────────────
% Run this section TWICE — once for a benign binary, once for malware.
% Save img_entropy as 'entropy_benign' and 'entropy_malware' then compare.

% Example (uncomment after loading both):
%
%   mse_val  = mean((entropy_benign(:) - entropy_malware(:)).^2);
%   psnr_val = psnr(mat2gray(entropy_malware), mat2gray(entropy_benign));
%   fprintf('MSE  = %.6f\n', mse_val);
%   fprintf('PSNR = %.2f dB\n', psnr_val);
%
%   figure('Name','6. Histogram comparison');
%   subplot(1,2,1); histogram(entropy_benign(:),  64); title('Benign entropy dist.');
%   subplot(1,2,2); histogram(entropy_malware(:), 64); title('Malware entropy dist.');

fprintf('\nPhase 2 complete. Proceed to metric comparison (section 6).\n');
