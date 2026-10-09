# Concept art for the README: the five places side by side, with the cast in front.
# Run from design/: python3 concept.py   -> ../docs/concept.png
import subprocess
B = '../assets/sprites/bg'; T = '/tmp/_concept'
def run(*a): subprocess.run(['convert', *map(str, a)], check=True)
subprocess.run(['mkdir', '-p', T, '../docs'], check=True)
places = [('cellar', 'THE CAVES'), ('australia', 'AUSTRALIA'), ('newzealand', 'NEW ZEALAND'), ('france', 'FRANCE'), ('uk', 'UNITED KINGDOM')]
W, H, PW = 1800, 900, 360
cmd = ['-size', f'{W}x{H}', 'xc:#0b0706']
for i, (name, label) in enumerate(places):
    run(f'{B}/{name}_far.png', f'{B}/{name}_mid.png', '-composite', '-gravity', 'center', '-crop', '640x1000+0+0', '+repage', '-resize', f'x{H}', '-gravity', 'center', '-crop', f'{PW-8}x{H}+0+0', '+repage', f'{T}/{name}.png')
    cmd += [f'{T}/{name}.png', '-geometry', f'+{i*PW+4}+0', '-composite']
cmd += ['-fill', '#0b0706', '-draw', f'rectangle 0,{H-150} {W},{H}']
for i, (name, label) in enumerate(places):
    cmd += ['-fill', '#f4e3c4', '-font', 'DejaVu-Sans-Bold', '-pointsize', '22', '-gravity', 'NorthWest', '-annotate', f'+{i*PW+22}+{H-46}', label]
# the cast, cut out of the approved sheet
cast = [('simon', '300x480+150+150', 300, 1.0), ('charm', '280x470+800+160', 660, 1.0), ('louis', '470x440+1190+185', 990, 0.8)]
for name, crop, x, s in cast:
    run('sheet3.png', '-crop', crop, '+repage', '-alpha', 'set', '-fuzz', '4%', '-fill', 'none', '-draw', 'matte 0,0 floodfill', '-resize', f'{int(s*100)}%', f'{T}/{name}.png')
    h = int(subprocess.run(['identify', '-format', '%h', f'{T}/{name}.png'], capture_output=True, text=True).stdout)
    cmd += [f'{T}/{name}.png', '-gravity', 'NorthWest', '-geometry', f'+{x}+{H-150-h+14}', '-composite']
for f, x, s in [('cherry', 60, 150), ('hopper', 1330, 190), ('beefsteak', 520, 230)]:
    run(f'../assets/sprites/tomatoes/{f}.png', '-trim', '+repage', '-resize', f'{s}x{s}', f'{T}/{f}.png')
    h = int(subprocess.run(['identify', '-format', '%h', f'{T}/{f}.png'], capture_output=True, text=True).stdout)
    cmd += [f'{T}/{f}.png', '-gravity', 'NorthWest', '-geometry', f'+{x}+{H-150-h+6}', '-composite']
run('../assets/sprites/tomatoes/boss.png', '-trim', '+repage', '-resize', 'x420', f'{T}/boss.png')
cmd += [f'{T}/boss.png', '-gravity', 'NorthWest', '-geometry', f'+{1540}+{H-150-420+10}', '-composite']
run(*cmd, '../docs/concept.png')
print('ok')
