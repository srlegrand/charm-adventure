# Bone-driven outlines: each limb is ONE closed shape whose edge is generated from the bone chain,
# the way Moho or Animate bind a vector outline to bones. No per-segment primitives.
import math
import shot as T
from shot import K, S, d, add
def curve(A, Kn, C, n=22, tension=.75):
    mid = ((A[0] + C[0]) / 2, (A[1] + C[1]) / 2); ctl = (Kn[0] + (Kn[0] - mid[0]) * tension, Kn[1] + (Kn[1] - mid[1]) * tension)
    return [((1-t)**2 * A[0] + 2*(1-t)*t * ctl[0] + t*t * C[0], (1-t)**2 * A[1] + 2*(1-t)*t * ctl[1] + t*t * C[1]) for t in [i / n for i in range(n + 1)]]
def outline(pts, width):
    L, Rr = [], []; n = len(pts) - 1
    for i, p in enumerate(pts):
        a = pts[max(i - 1, 0)]; b = pts[min(i + 1, n)]; dx, dy = b[0] - a[0], b[1] - a[1]; l = math.hypot(dx, dy) or 1
        w = width(i / n); L.append((p[0] - dy / l * w, p[1] + dx / l * w)); Rr.append((p[0] + dy / l * w, p[1] - dx / l * w))
    return L, Rr
def smooth(points):
    # closed Catmull-Rom through the points, as cubic Beziers
    n = len(points); o = f'M{points[0][0]:.1f},{points[0][1]:.1f}'
    for i in range(n):
        p0, p1, p2, p3 = points[i - 1], points[i], points[(i + 1) % n], points[(i + 2) % n]
        o += f' C{p1[0]+(p2[0]-p0[0])/6:.1f},{p1[1]+(p2[1]-p0[1])/6:.1f} {p2[0]-(p3[0]-p1[0])/6:.1f},{p2[1]-(p3[1]-p1[1])/6:.1f} {p2[0]:.1f},{p2[1]:.1f}'
    return o + 'Z'
def limb(A, Kn, C, width, fill, upto=None, fill2=None, tension=.75):
    pts = curve(A, Kn, C, tension=tension); L, Rr = outline(pts, width)
    o = f'<path d="{smooth(L + Rr[::-1])}" fill="{fill}" {S()}/>'
    if upto:   # second colour over the first part of the same outline (a sleeve), no new silhouette
        k = int(len(pts) * upto); o += f'<path d="{smooth(L[:k] + Rr[:k][::-1])}" fill="{fill2}" {S()}/>'
    return o
leg_w = lambda w0, w1: (lambda t: w0 + (w1 - w0) * t + 5 * math.sin(math.pi * min(t / .45, 1)) * (1 - t) + 3 * math.exp(-((t - .68) / .12) ** 2))
arm_w = lambda w0, w1: (lambda t: w0 + (w1 - w0) * t + 3 * math.exp(-((t - .62) / .12) ** 2) + 9 * math.exp(-((t - 1) / .09) ** 2))

def simon(pelvis, spine, legs, arms, weapon_rot):
    tf, sf, tb, sb = legs; uf, lf, ub, lb = arms; pel = pelvis; sk = '#f6dcc0'; tee = '#f6efe0'; jean = '#3f4a72'
    kf = add(pel, d(tf, 95)); ff = add(kf, d(sf, 100)); kb = add(pel, d(tb, 95)); fb = add(kb, d(sb, 100))
    neck = add(pel, d(180 - spine, 140)); sh = add(pel, d(180 - spine, 122)); hd = add(neck, d(180 - spine, 46)); hd = (hd[0] + 10, hd[1])
    ef = add(sh, d(uf, 76)); hf = add(ef, d(lf, 72)); eb = add(sh, d(ub, 76)); hb = add(eb, d(lb, 72))
    foot = lambda p, a: f'<g transform="translate({p[0]:.1f},{p[1]:.1f}) rotate({a})">{T.SHOE}</g>'
    o = limb(sh, eb, hb, arm_w(14, 10), sk, .5, tee)
    o += limb(pel, kb, fb, leg_w(24, 12), jean) + foot(fb, -sb * .6)
    # torso: one shape from hips to shoulders, bowed along the spine
    belly = add(pel, d(180 - spine + 14, 70)); tp = curve(pel, belly, neck, tension=.5)
    L, Rr = outline(tp, lambda t: 30 + 6 * math.sin(math.pi * t) - 8 * t * t)
    o += f'<clipPath id="tc"><path d="{smooth(L + Rr[::-1])}"/></clipPath><path d="{smooth(L + Rr[::-1])}" fill="{tee}" {S()}/><g clip-path="url(#tc)">'
    for i in range(2, len(tp) - 2, 3):
        a = tp[i - 1]; b = tp[i + 1]; dx, dy = b[0] - a[0], b[1] - a[1]; l = math.hypot(dx, dy); nx, ny = -dy / l * 60, dx / l * 60
        o += f'<path d="M{tp[i][0]-nx:.1f},{tp[i][1]-ny:.1f} Q{tp[i][0]+dx*.25:.1f},{tp[i][1]+dy*.25:.1f} {tp[i][0]+nx:.1f},{tp[i][1]+ny:.1f}" stroke="#27305c" stroke-width="9" fill="none"/>'
    o += f'</g><path d="{smooth(L + Rr[::-1])}" fill="none" {S()}/>'
    o += limb(pel, kf, ff, leg_w(25, 12), jean) + foot(ff, -sf * .6)
    o += T.tube(neck, add(neck, d(180 - spine, 20)), 12, 11, sk) + T.head(hd, 42, sk, spine * .3 + 4, T.SIMON['face'])
    o += f'<g transform="translate({hf[0]:.1f},{hf[1]:.1f}) rotate({weapon_rot})">{T.CROWBAR}</g>'
    o += limb(sh, ef, hf, arm_w(15, 10), sk, .5, tee)
    return o
if __name__ == '__main__':
    pose = ((0, -132), 42, (70, 5, -55, -40), (96, 100, -70, -105))
    old = T.human(*pose, T.SIMON, (T.CROWBAR, 96), 'f', 4); new = simon(*pose, 96)
    lab = lambda x, t: f'<text x="{x}" y="60" text-anchor="middle" font-size="28" font-weight="800" letter-spacing="3" fill="#8a2a3a" font-family="DejaVu Sans">{t}</text>'
    body = f'<rect width="1500" height="560" fill="#3a2817"/>' + lab(380, 'BEFORE: SEGMENTS') + lab(1130, 'AFTER: ONE OUTLINE PER LIMB') + T.g(320, 500, old, 1.25) + T.g(1070, 500, new, 1.25)
    open('limb.svg', 'w').write(f'<svg xmlns="http://www.w3.org/2000/svg" width="1500" height="560">{body}</svg>')
