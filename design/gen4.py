# Design pass 4: jointed figures posed from a skeleton, after Simon's sketches.
# Every limb segment is its own shape, so the same skeleton can drive the in-game rig.
import math, random
P = '#3b1436'; DK = '#2a0f2c'
def S(w=5): return f'stroke="{P}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round"'
def d(a, l): r = math.radians(a); return (math.sin(r) * l, math.cos(r) * l)   # 0 = down, +90 = forward
def add(p, q): return (p[0] + q[0], p[1] + q[1])
def seg(p, q, w0, w1, fill, sh=None):
    dx, dy = q[0] - p[0], q[1] - p[1]; L = math.hypot(dx, dy) or 1; nx, ny = -dy / L, dx / L
    pts = [(p[0] + nx * w0, p[1] + ny * w0), (q[0] + nx * w1, q[1] + ny * w1), (q[0] - nx * w1, q[1] - ny * w1), (p[0] - nx * w0, p[1] - ny * w0)]
    quad = f'M{pts[0][0]:.1f},{pts[0][1]:.1f} L{pts[1][0]:.1f},{pts[1][1]:.1f} L{pts[2][0]:.1f},{pts[2][1]:.1f} L{pts[3][0]:.1f},{pts[3][1]:.1f}Z'
    caps = f'<circle cx="{p[0]:.1f}" cy="{p[1]:.1f}" r="{w0}" FILL/><circle cx="{q[0]:.1f}" cy="{q[1]:.1f}" r="{w1}" FILL/><path d="{quad}" FILL/>'
    o = caps.replace('FILL', f'fill="{fill}" {S(10)}') + caps.replace('FILL', f'fill="{fill}"')
    if sh: o += f'<path d="M{pts[2][0]:.1f},{pts[2][1]:.1f} L{pts[3][0]:.1f},{pts[3][1]:.1f} L{(p[0]+pts[3][0])/2:.1f},{(p[1]+pts[3][1])/2:.1f} L{(q[0]+pts[2][0])/2:.1f},{(q[1]+pts[2][1])/2:.1f}Z" fill="{sh}" opacity=".5"/>'
    return o
def shoe(p, a, col, L=50):
    return f'<g transform="translate({p[0]:.1f},{p[1]:.1f}) rotate({a})"><path d="M-16,-12 C-18,6 -14,12 0,12 L{L},12 C{L+8},10 {L+4},-2 {L-8},-4 L10,-14Z" fill="{col}" {S()}/><path d="M-14,8 L{L+2},8" stroke="#f6efe0" stroke-width="4"/></g>'
def head(c, r, skin, tilt=0, extra='', behind=''):
    x, y = c
    face = (f'M{-.9*r},{.2*r} C{-1.1*r},{-.9*r} {.2*r},{-1.3*r} {.75*r},{-.7*r} L{.86*r},{-.3*r} L{1.14*r},{.08*r} L{.9*r},{.2*r} L{.98*r},{.4*r} L{.82*r},{.5*r} '
            f'L{.8*r},{.86*r} C{.4*r},{1.08*r} {0},{.9*r} {-.2*r},{.55*r} C{-.6*r},{.6*r} {-.85*r},{.45*r} {-.9*r},{.2*r}Z')
    return (f'<g transform="translate({x:.1f},{y:.1f}) rotate({tilt})">{behind}<path d="{face}" fill="{skin}" {S()}/>'
            f'<path d="M{.2*r},{-.42*r} L{.92*r},{-.2*r}" stroke="{P}" stroke-width="9" stroke-linecap="round"/>'
            f'<path d="M{.42*r},{-.2*r} L{.8*r},{-.1*r} L{.5*r},{-.02*r}Z" fill="{P}"/>'
            f'<path d="M{.62*r},{.5*r} L{.86*r},{.46*r}" stroke="{P}" stroke-width="4" stroke-linecap="round"/>{extra}</g>')

def simon():
    sk = '#f3d2b3'; tee = 'url(#br)'; jean = '#3a4468'; jsh = '#232a48'
    pel = (0, -177)
    kf = add(pel, d(45, 95)); ff = add(kf, d(10, 100))            # front leg
    kb = add(pel, d(-40, 95)); fb = add(kb, d(-23, 100))          # back leg
    up = (math.sin(math.radians(25)), -math.cos(math.radians(25)))
    neck = (pel[0] + up[0] * 150, pel[1] + up[1] * 150)
    hd = (neck[0] + 26, neck[1] - 40)
    sh = (neck[0] - 6, neck[1] + 16)
    ef = add(sh, d(20, 78)); hf = add(ef, d(55, 74))              # front arm, fist forward
    eb = add(sh, d(-125, 74)); hb = add(eb, d(172, 70))           # weapon arm, raised behind the head
    o = f'<ellipse cx="-20" cy="6" rx="190" ry="12" fill="{P}" opacity=".12"/>'
    bar = f'<g transform="translate({hb[0]:.1f},{hb[1]:.1f}) rotate(-38)"><path d="M-8,170 L8,170 L8,-120 q4,-34 34,-30 l-2,14 q-18,-2 -18,18 L8,-100 L-8,-100Z" fill="#e2452e" {S()}/><path d="M0,160 L0,-110" stroke="#ff9a7a" stroke-width="3"/></g>'
    o += bar + seg(sh, eb, 15, 12, sk, '#d9a984') + seg(eb, hb, 12, 10, sk, '#d9a984') + f'<circle cx="{hb[0]:.1f}" cy="{hb[1]:.1f}" r="15" fill="{sk}" {S()}/>'
    o += seg(pel, kb, 24, 18, jean, jsh) + seg(kb, fb, 18, 13, jean, jsh) + shoe(fb, 8, DK)
    torso = f'M{pel[0]-30},{pel[1]+8} L{pel[0]+30},{pel[1]+4} L{neck[0]+34:.1f},{neck[1]+22:.1f} L{neck[0]+6:.1f},{neck[1]:.1f} L{neck[0]-34:.1f},{neck[1]+10:.1f} Z'
    o += f'<path d="{torso}" fill="{tee}" {S()}/><path d="M{pel[0]+30},{pel[1]+4} L{neck[0]+34:.1f},{neck[1]+22:.1f} L{neck[0]+10:.1f},{neck[1]+40:.1f} L{pel[0]+6},{pel[1]+6}Z" fill="{P}" opacity=".22"/>'
    o += seg(pel, kf, 25, 19, jean, jsh) + seg(kf, ff, 19, 13, jean, jsh) + shoe(ff, 0, DK)
    o += seg((neck[0] + 4, neck[1] + 6), (hd[0] - 8, hd[1] + 20), 13, 12, sk)
    goat = f'<path d="M22,18 L33,36 C20,46 6,40 2,28Z" fill="#4a3a52"/><ellipse cx="-12" cy="2" rx="8" ry="12" fill="#e2b894" {S(4)}/><circle cx="-14" cy="18" r="7" fill="none" stroke="#f2b24a" stroke-width="4"/><path d="M-10,-44 q22,-12 44,2" stroke="#fff" stroke-width="5" fill="none" opacity=".7" stroke-linecap="round"/>'
    o += head(hd, 44, sk, 8, goat)
    o += seg(sh, ef, 16, 13, tee) + seg(ef, hf, 12, 10, sk, '#d9a984') + f'<path transform="translate({hf[0]:.1f},{hf[1]:.1f}) rotate(-30)" d="M-14,-12 h26 q8,2 6,14 q-2,12 -16,12 h-14 q-8,-12 -2,-26Z" fill="{sk}" {S()}/>'
    return o

def charm():
    sk = '#a9683f'; skd = '#8a5230'; hair = '#2a1024'; leg = DK
    pel = (0, -214)
    kf = add(pel, d(28, 110)); ff = add(kf, d(22, 115))
    kb = add(pel, d(-38, 110)); fb = add(kb, d(-24, 118))
    up = (math.sin(math.radians(6)), -math.cos(math.radians(6)))
    neck = (pel[0] + up[0] * 150, pel[1] + up[1] * 150)
    hd = (neck[0] + 18, neck[1] - 42)
    sh = (neck[0] - 4, neck[1] + 16)
    eb = add(sh, d(-38, 82)); hb = add(eb, d(-52, 84))            # near arm, thrown back
    ef = add(sh, d(62, 76)); hf = add(ef, d(168, 70))             # sword arm, hand up by the head
    o = f'<ellipse cx="-20" cy="6" rx="200" ry="12" fill="{P}" opacity=".12"/>'
    sword = (f'<g transform="translate({hf[0]:.1f},{hf[1]:.1f}) rotate(-62)"><path d="M-9,-26 L-13,-300 L0,-336 L13,-300 L9,-26Z" fill="#eaf6ff" {S()}/><path d="M0,-30 L0,-322" stroke="#9fd0ea" stroke-width="3"/>'
             f'<path d="M-40,-30 q40,-18 80,0 q-40,14 -80,0Z" fill="#f2b24a" {S()}/><path d="M-7,-22 h14 v52 h-14Z" fill="{DK}" {S(4)}/><circle cx="0" cy="40" r="12" fill="#f2b24a" {S()}/></g>')
    o += seg(pel, kb, 22, 16, leg) + seg(kb, fb, 16, 9, leg) + f'<path transform="translate({fb[0]:.1f},{fb[1]:.1f}) rotate(40)" d="M-9,-10 L9,-10 L14,34 L-4,36Z" fill="{leg}" {S()}/>'
    o += sword + seg(sh, ef, 13, 11, sk, skd) + seg(ef, hf, 11, 9, sk, skd)
    o += seg(pel, kf, 23, 17, leg) + seg(kf, ff, 17, 10, leg) + f'<path transform="translate({ff[0]:.1f},{ff[1]:.1f}) rotate(-58)" d="M-9,-10 L9,-10 L14,40 L-4,42Z" fill="{leg}" {S()}/>'
    tunic = f'M{neck[0]-36:.1f},{neck[1]+8:.1f} L{neck[0]+4:.1f},{neck[1]:.1f} L{neck[0]+30:.1f},{neck[1]+18:.1f} L{pel[0]+50},{pel[1]+40} L{pel[0]+74},{pel[1]+86} L{pel[0]+20},{pel[1]+70} L{pel[0]-20},{pel[1]+96} L{pel[0]-64},{pel[1]+62} L{pel[0]-34},{pel[1]+10} Z'
    o += f'<path d="{tunic}" fill="url(#ch)" {S()}/><path d="M{neck[0]+30:.1f},{neck[1]+18:.1f} L{pel[0]+50},{pel[1]+40} L{pel[0]+74},{pel[1]+86} L{pel[0]+20},{pel[1]+70} L{pel[0]+12},{pel[1]}Z" fill="{P}" opacity=".28"/>'
    o += seg((neck[0] + 2, neck[1] + 6), (hd[0] - 8, hd[1] + 22), 11, 10, sk)
    R = random.Random(5); pts = []
    for i in range(15):
        a = math.radians(150 + i * 15); r = 44 * (1.75 if i % 2 == 0 else 1.0) * R.uniform(.9, 1.25)
        pts.append(f'{math.sin(a)*r*-1 - 10:.1f},{math.cos(a)*r*-1 + 6:.1f}')
    spikes = f'<path d="M{" L".join(pts)}Z" fill="{hair}" {S()}/>'
    fringe = f'<path d="M-44,-4 C-46,-56 30,-64 38,-28 C14,-40 -6,-30 -16,-4 L-22,30 L-40,24Z" fill="{hair}" {S(4)}/><path d="M-6,-50 C-30,-30 -40,0 -66,26" stroke="#d9c08a" stroke-width="7" fill="none" stroke-linecap="round"/>'
    o += head(hd, 42, sk, 4, fringe, spikes)
    o += f'<circle cx="{hf[0]:.1f}" cy="{hf[1]:.1f}" r="14" fill="{sk}" {S()}/>'
    o += seg(sh, eb, 13, 11, sk, skd) + seg(eb, hb, 11, 9, sk, skd) + f'<path transform="translate({hb[0]:.1f},{hb[1]:.1f}) rotate(48)" d="M-10,-8 L10,-8 L12,14 L4,34 L-2,16 L-12,26Z" fill="{sk}" {S()}/>'
    return o

def tomato():
    r = 150; cy = -r - 46
    o = f'<ellipse cx="0" cy="6" rx="170" ry="12" fill="{P}" opacity=".12"/>'
    o += f'<path d="M-60,-70 L-96,-22 M40,-56 L62,-14" stroke="{P}" stroke-width="16" stroke-linecap="round"/><path d="M-60,-70 L-96,-22 M40,-56 L62,-14" stroke="{DK}" stroke-width="8" stroke-linecap="round"/>'
    o += shoe((-96, -14), 180 + 14, DK, 40).replace('rotate(194)', 'rotate(14) scale(-1,1)') + shoe((62, -6), 0, DK, 40).replace('rotate(0)', 'scale(-1,1)')
    o += f'<g transform="translate(0,{cy})">'
    o += f'<path d="M-150,0 C-156,-90 -80,-152 0,-150 C90,-150 152,-84 150,0 C152,90 84,150 0,150 C-70,152 -118,120 -130,70 C-150,60 -158,30 -150,0Z" fill="#d8342c" {S(6)}/>'
    o += f'<path d="M150,0 C152,90 84,150 0,150 C-30,150 -60,144 -84,128 C0,130 96,80 104,-80 C134,-56 150,-30 150,0Z" fill="#9c1c26"/>'
    o += f'<path d="M-96,-96 C-70,-128 -30,-140 10,-136 C-30,-124 -66,-108 -96,-96Z" fill="#f58a6a"/>'
    o += f'<path d="M10,-146 C-10,-170 -44,-160 -60,-150 C-30,-150 -20,-140 -6,-132 C8,-120 40,-126 58,-140 C40,-150 26,-148 10,-146Z" fill="#3f7a3a" {S()}/>'
    o += f'<path d="M4,-146 C-2,-190 10,-214 44,-206 C60,-200 56,-180 40,-184 C22,-188 20,-170 24,-146Z" fill="#3f7a3a" {S()}/>'
    # brows, eyes, nose bump, snarl with underbite
    o += f'<path d="M-134,-60 C-110,-84 -84,-80 -62,-52 L-70,-40 C-90,-56 -112,-56 -130,-44Z M-34,-52 C0,-84 40,-80 66,-56 L58,-40 C34,-56 4,-54 -22,-36Z" fill="{P}"/>'
    o += f'<path d="M-124,-40 C-108,-50 -88,-48 -72,-34 C-90,-22 -110,-24 -124,-40Z M-22,-32 C4,-50 34,-48 54,-34 C30,-14 0,-14 -22,-32Z" fill="#f6efe0" {S(4)}/><circle cx="-96" cy="-36" r="8" fill="{P}"/><circle cx="14" cy="-32" r="10" fill="{P}"/>'
    o += f'<path d="M-150,-6 C-170,-4 -172,24 -150,28" fill="#d8342c" {S(5)}/>'
    o += f'<path d="M-132,62 C-110,34 -40,22 30,40 C50,48 50,66 30,70 C-20,60 -80,70 -112,96Z" fill="{DK}" {S()}/>'
    o += f'<path d="M-112,84 L-98,58 L-84,78 L-66,52 L-52,72 L-32,50 L-18,68 L2,50 L14,66" fill="#f6efe0" {S(4)}/>'
    o += f'<path d="M-128,104 C-90,84 -30,78 20,90" fill="none" {S(5)}/>'
    o += '</g>'
    return o

DEFS = f'''<defs>
<pattern id="br" width="10" height="22" patternUnits="userSpaceOnUse" patternTransform="rotate(25)"><rect width="10" height="22" fill="#f6efe0"/><rect width="10" height="10" fill="#27305c"/></pattern>
<pattern id="ch" width="22" height="22" patternUnits="userSpaceOnUse" patternTransform="rotate(12)"><rect width="22" height="22" fill="#a8262a"/><rect x="5" y="5" width="8" height="8" fill="none" stroke="#f1d6a8" stroke-width="2.2"/><circle cx="17" cy="17" r="1.8" fill="#f1d6a8"/></pattern></defs>'''
def g(x, y, b, s=1.0): return f'<g transform="translate({x},{y}) scale({s})">{b}</g>'
def label(x, y, t): return f'<text x="{x}" y="{y}" text-anchor="middle" font-size="30" font-weight="800" letter-spacing="3" fill="#8a2a3a">{t}</text>'
if __name__ == '__main__':
    p = [DEFS, '<rect width="1800" height="800" fill="#faf5ee"/>', g(320, 660, simon(), 1.15), label(300, 730, 'SIMON'), g(900, 660, charm(), 1.05), label(880, 730, 'CHARM'), g(1470, 660, tomato(), 1.1), label(1470, 730, 'TOMATO')]
    open('sheet4.svg', 'w').write(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1800 800" width="1800" height="800" font-family="DejaVu Sans, Arial, sans-serif">{"".join(p)}</svg>')
