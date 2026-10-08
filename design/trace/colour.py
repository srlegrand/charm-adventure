# Colours the traced sketch: flat colour under Simon's own ink line. Coordinates are sketch pixels.
import numpy as np, cv2
from scipy import ndimage as ndi
ink = np.load('ink.npy'); reg = np.load('reg.npy'); H, W = ink.shape
def hexc(h): h = h.lstrip('#'); return tuple(int(h[i:i+2], 16) for i in (4, 2, 0))
flat = np.zeros((H, W, 4), np.uint8)
def paint(mask, col): flat[mask] = (*hexc(col), 255)
def at(*pts, grow=7):
    m = np.zeros((H, W), bool)
    for x, y in pts:
        l = reg[y, x]
        if l == 0:
            ys, xs = np.where(reg[y-12:y+13, x-12:x+13] > 0)
            if len(ys) == 0: print('no region at', x, y); continue
            l = reg[y-12+ys[0], x-12+xs[0]]
        m |= reg == l
    return ndi.binary_dilation(m, iterations=grow)
def poly(*pts):
    m = np.zeros((H, W), np.uint8); cv2.fillPoly(m, [np.array(pts, np.int32)], 1); return m.astype(bool)
SK, SKC = '#f3d2b3', '#a9683f'; DK = '#2a0f2c'
# Simon
paint(poly((395,775),(520,765),(665,830),(605,1000),(555,1055),(485,1040),(590,960),(590,862),(470,855),(415,915),(360,995),(330,1020),(255,1075),(185,1035),(350,930),(385,820)), '#3a4468')
paint(poly((385,590),(480,515),(530,530),(620,640),(520,730),(495,760),(385,775),(405,680)), '#f6efe0')
paint(poly((92,1092),(150,1070),(250,1085),(262,1120),(180,1146),(100,1126)), DK)
paint(poly((455,1130),(480,1075),(560,1075),(660,1110),(650,1146),(590,1166),(520,1160)), DK)
paint(at((600,420)), SK); paint(at((285,600), (335,448)), SK); paint(at((700,780)), SK)
paint(poly((232,470),(262,430),(310,400),(325,430),(290,480),(245,490)), SK)
paint(poly((640,535),(690,525),(700,545),(670,560)), '#4a3a52')
paint(at((425,330)), '#e2452e')
# Charm
paint(poly((1105,480),(1140,400),(1220,355),(1300,360),(1330,420),(1290,432),(1240,500),(1232,590),(1170,600),(1120,560)), '#5a3350')
paint(poly((1210,905),(1420,940),(1380,962),(1350,1000),(1220,990)), DK)
paint(at((1135,1050), (1430,1100)), DK)
paint(poly((900,1290),(925,1205),(990,1190),(970,1240),(940,1292)), DK); paint(poly((1500,1290),(1540,1280),(1600,1318),(1510,1300)), DK)
paint(at((1333,830), (1278,730), (1210,838), (1158,910)), '#a8262a'); paint(poly((1255,700),(1300,690),(1390,660),(1400,700),(1300,712)), '#a8262a')
paint(at((1295,495), (1320,618), (1170,680), (978,893)), SKC); paint(poly((1385,642),(1400,628),(1515,668),(1528,555),(1548,560),(1538,702),(1505,702)), SKC)
paint(poly((1500,470),(1545,430),(1610,480),(1600,530),(1535,540)), SKC)
paint(at((1112,320), (1095,343), (1393,414), (1388,445)), '#eaf6ff'); paint(at((1458,460), (1537,388), (1580,523)), '#f2b24a')
# Tomato
paint(at((2300,700), grow=9), '#d8342c'); paint(at((2400,888), (2250,988), (2130,990), (2075,940)), '#9c1c26')
paint(at((2210,612), (2170,728), (2148,605)), '#f6efe0'); paint(at((2240,574), (2142,700), (2025,588), (2010,550)), '#b8242a')
paint(at((2262,360), (2225,383)), '#3f7a3a'); paint(poly((2205,470),(2205,390),(2225,345),(2280,345),(2292,385),(2250,395),(2255,470)), '#3f7a3a')
paint(at((2022,934), (2280,1057), (2284,1100)), DK); paint(poly((1910,960),(1940,950),(1985,995),(1930,990)), DK)
flat[..., 3] = cv2.GaussianBlur(flat[..., 3], (0, 0), 1.2)
line = np.zeros((H, W, 4), np.uint8); line[ink] = (*hexc('#2b1230'), 255); line[..., 3] = cv2.GaussianBlur(line[..., 3], (0, 0), 0.8)
def over(bg, fg):
    a = fg[..., 3:4].astype(np.float32) / 255; out = bg.astype(np.float32)
    out[..., :3] = fg[..., :3] * a + out[..., :3] * (1 - a); out[..., 3:4] = np.maximum(out[..., 3:4], fg[..., 3:4]); return out.astype(np.uint8)
art = over(flat, line); cv2.imwrite('traced.png', art)
paper = np.full((H, W, 4), (238, 245, 250, 255), np.uint8); cv2.imwrite('traced_sheet.png', over(paper, art)[110:1350, 20:2540, :3])
for name, (x0, x1) in {'simon': (40, 860), 'charm': (860, 1720), 'tomato': (1880, 2520)}.items():
    cv2.imwrite(f'{name}.png', art[120:1340, x0:min(x1, W)])
