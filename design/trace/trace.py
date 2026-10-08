# Traces Simon's sketch: extracts the ink line, finds the enclosed regions, and labels them for colouring.
import numpy as np, cv2, sys
from scipy import ndimage as ndi
img = cv2.imread('sketch.jpg'); g = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY).astype(np.float32)
bg = cv2.GaussianBlur(g, (0, 0), 40); n = g / bg                      # flatten the lighting
ink = n < 0.55                                                         # black pen only; the blue pencil is lighter
ink = ndi.binary_opening(ink, iterations=1) | (n < 0.42)
lab, k = ndi.label(ink); sizes = ndi.sum(ink, lab, range(1, k + 1))
ink = np.isin(lab, 1 + np.where(sizes > 25)[0])                        # drop specks
np.save('ink.npy', ink)
barrier = ndi.binary_dilation(ink, iterations=4)                       # close small gaps in the line
sil = ndi.binary_fill_holes(ndi.binary_dilation(ink, iterations=9)); sil = ndi.binary_erosion(sil, iterations=7)
free = sil & ~barrier
reg, k = ndi.label(free); sizes = ndi.sum(free, reg, range(1, k + 1))
np.save('reg.npy', reg); np.save('sil.npy', sil)
rng = np.random.RandomState(3); out = np.full(img.shape, 255, np.uint8)
big = [i + 1 for i in np.argsort(-sizes) if sizes[i] > 350]
for i in big:
    out[reg == i] = rng.randint(90, 230, 3)
out[ink] = 0
for i in big:
    cy, cx = ndi.center_of_mass(reg == i)
    ys, xs = np.where(reg == i); j = np.argmin((ys - cy) ** 2 + (xs - cx) ** 2)
    cv2.putText(out, str(i), (int(xs[j]) - 12, int(ys[j]) + 6), cv2.FONT_HERSHEY_SIMPLEX, 0.6, (0, 0, 255), 2)
cv2.imwrite('regions.png', out); print(len(big), 'regions')
