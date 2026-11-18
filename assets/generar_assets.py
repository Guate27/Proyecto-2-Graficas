"""
Genera las texturas de los bloques (16x16, estilo pixel art) y las seis caras
del skybox como archivos BMP de 24 bits. Solo usa la librería estándar de
Python. Se ejecuta una vez desde la raíz del proyecto:

    python assets/generar_assets.py

El programa en Rust no depende de este script: solo lee los BMP generados.
"""

import math
import os
import random
import struct

RAIZ = os.path.dirname(os.path.abspath(__file__))


# --------------------------------------------------------------------------
# Escritura de BMP de 24 bits (filas de abajo hacia arriba, con relleno a 4)
# --------------------------------------------------------------------------
def guardar_bmp(ruta, ancho, alto, pixeles):
    relleno = (4 - (ancho * 3) % 4) % 4
    tam_fila = ancho * 3 + relleno
    tam_datos = tam_fila * alto
    cabecera = struct.pack("<2sIHHI", b"BM", 54 + tam_datos, 0, 0, 54)
    info = struct.pack("<IiiHHIIiiII", 40, ancho, alto, 1, 24, 0, tam_datos, 2835, 2835, 0, 0)
    datos = bytearray()
    for y in range(alto - 1, -1, -1):
        for x in range(ancho):
            r, g, b = pixeles[y * ancho + x]
            datos += bytes((clamp8(b), clamp8(g), clamp8(r)))
        datos += b"\x00" * relleno
    with open(ruta, "wb") as f:
        f.write(cabecera + info + datos)


def clamp8(v):
    return max(0, min(255, int(round(v))))


def hexrgb(h):
    return ((h >> 16) & 255, (h >> 8) & 255, h & 255)


def mezclar(a, b, t):
    return tuple(a[i] + (b[i] - a[i]) * t for i in range(3))


def escalar(c, k):
    return tuple(v * k for v in c)


# --------------------------------------------------------------------------
# Texturas de bloques 16x16
# --------------------------------------------------------------------------
N = 16


def textura(nombre, funcion, semilla):
    rnd = random.Random(semilla)
    pix = [funcion(x, y, rnd) for y in range(N) for x in range(N)]
    guardar_bmp(os.path.join(RAIZ, "textures", nombre + ".bmp"), N, N, pix)


def elegir(rnd, paleta):
    return hexrgb(rnd.choice(paleta))


# Pasto turquesa con florecitas rosadas y amarillas.
def pasto_arriba(x, y, rnd):
    r = rnd.random()
    if r < 0.025:
        return hexrgb(0xF28DB2)
    if r < 0.045:
        return hexrgb(0xF6D55C)
    return elegir(rnd, [0x3FA77F, 0x3FA77F, 0x4BB58B, 0x36966F, 0x58C095, 0x339068])


# Tierra color terracota, con piedritas claras.
def tierra(x, y, rnd):
    c = elegir(rnd, [0xB5653F, 0xB5653F, 0xA45A37, 0xC27349, 0x9A5232])
    if rnd.random() < 0.05:
        c = hexrgb(0xD9A27C)
    return c


ALTURA_PASTO = None


# Lado del pasto: el musgo "chorrea" hacia abajo en franjas irregulares.
def pasto_lado(x, y, rnd):
    global ALTURA_PASTO
    if ALTURA_PASTO is None:
        r2 = random.Random(77)
        ALTURA_PASTO = [r2.choice([2, 3, 3, 4, 6, 7]) for _ in range(N)]
    if y < ALTURA_PASTO[x]:
        return elegir(rnd, [0x3FA77F, 0x36966F, 0x4BB58B])
    return tierra(x, y, rnd)


# Piedra en ladrillos rectangulares color pizarra azulada.
def piedra(x, y, rnd):
    fila = y // 4
    desfase = 4 if fila % 2 else 0
    if y % 4 == 3 or (x + desfase) % 8 == 7:
        return hexrgb(0x3E4A5C)
    tonos = [0x6D7C93, 0x75849A, 0x66748A, 0x7A8AA2]
    base = hexrgb(tonos[(fila + (x + desfase) // 8) % 4])
    return escalar(base, 1 + rnd.uniform(-0.06, 0.06))


# Arena durazno con ondas de viento.
def arena(x, y, rnd):
    onda = math.sin(x * 0.8 + y * 0.35) * 0.5 + 0.5
    base = mezclar(hexrgb(0xF2C9A0), hexrgb(0xE6B285), onda)
    return escalar(base, 1 + rnd.uniform(-0.03, 0.03))


# Corteza rojiza con surcos en diagonal.
def tronco_lado(x, y, rnd):
    c = hexrgb(0x7A3B2E)
    if (x + y) % 5 == 0:
        c = hexrgb(0x4E241C)
    elif (x + y) % 5 == 2:
        c = hexrgb(0x8C4A3A)
    return escalar(c, 1 + rnd.uniform(-0.06, 0.06))


# Corte del tronco: anillos claros y oscuros alternados.
def tronco_arriba(x, y, rnd):
    if x in (0, N - 1) or y in (0, N - 1):
        return tronco_lado(x, y, rnd)
    d = math.hypot(x + 0.5 - N / 2, y + 0.5 - N / 2)
    claro, oscuro = hexrgb(0xE0A77F), hexrgb(0xC4855E)
    c = claro if int(d * 0.8) % 2 == 0 else oscuro
    return escalar(c, 1 + rnd.uniform(-0.04, 0.04))


# Tablones en espiga (parqué) color miel.
def tablones(x, y, rnd):
    bloque = ((x // 4) + (y // 4)) % 2
    if bloque == 0:
        linea = (y % 4 == 0)
    else:
        linea = (x % 4 == 0)
    if linea:
        return hexrgb(0x8A5A2B)
    base = hexrgb(0xD9A15B) if bloque else hexrgb(0xCC9350)
    return escalar(base, 1 + rnd.uniform(-0.05, 0.05))


# Hojas de otoño: naranjas, rojas y doradas.
def hojas(x, y, rnd):
    if rnd.random() < 0.10:
        return hexrgb(0x8C2F1B)
    return elegir(rnd, [0xE0782F, 0xD2652A, 0xF0A43A, 0xC9502A, 0xE88C34])


# Agua turquesa con líneas de luz (cáusticas).
def agua(x, y, rnd):
    onda = math.sin(x * 0.9 + math.sin(y * 0.7) * 2.0) * 0.5 + 0.5
    base = mezclar(hexrgb(0x1E9AA8), hexrgb(0x3CC0C8), onda)
    if abs(math.sin(x * 0.55 + y * 0.9 + math.sin(x * 0.4))) < 0.12:
        base = hexrgb(0x9FE8EA)
    return escalar(base, 1 + rnd.uniform(-0.03, 0.03))


# Vitral: cuatro paneles separados por varillas de plomo.
def vidrio(x, y, rnd):
    if x in (0, N - 1) or y in (0, N - 1) or x in (7, 8) or y in (7, 8):
        return hexrgb(0x5B6470)
    if x - y in (2, 3):
        return hexrgb(0xFFFFFF)
    return hexrgb(0xF1F7EC)


# Cristal de amatista: facetas violetas con destellos.
def amatista(x, y, rnd):
    if x in (0, N - 1) or y in (0, N - 1):
        return hexrgb(0x5B2A86)
    faceta = (x + y) // 4 % 3
    c = [hexrgb(0x9B59D0), hexrgb(0xB57EE5), hexrgb(0x8446BD)][faceta]
    if rnd.random() < 0.05:
        c = hexrgb(0xF3E6FF)
    return c


# --------------------------------------------------------------------------
# Skybox: seis caras generadas a partir de la dirección de cada píxel, así
# las orillas de caras vecinas coinciden.
# --------------------------------------------------------------------------
DIR_SOL = None


def normalizar(v):
    l = math.sqrt(sum(c * c for c in v))
    return tuple(c / l for c in v)


DIR_SOL = normalizar((0.7, 0.62, -0.3))

PERM = list(range(256))
random.Random(3).shuffle(PERM)
PERM += PERM


def ruido2(x, y):
    xi, yi = int(math.floor(x)) & 255, int(math.floor(y)) & 255
    xf, yf = x - math.floor(x), y - math.floor(y)
    u, v = xf * xf * (3 - 2 * xf), yf * yf * (3 - 2 * yf)

    def h(i, j):
        return PERM[PERM[i] + j] / 255.0

    a = h(xi, yi) + (h(xi + 1, yi) - h(xi, yi)) * u
    b = h(xi, yi + 1) + (h(xi + 1, yi + 1) - h(xi, yi + 1)) * u
    return a + (b - a) * v


def fbm(x, y):
    total, amp, freq = 0.0, 0.5, 1.0
    for _ in range(5):
        total += amp * ruido2(x * freq, y * freq)
        amp *= 0.5
        freq *= 2.0
    return total


def suave(a, b, t):
    t = max(0.0, min(1.0, (t - a) / (b - a)))
    return t * t * (3 - 2 * t)


def color_cielo(d):
    x, y, z = d
    cenit, horizonte = (0.23, 0.47, 0.86), (0.80, 0.89, 0.98)
    if y >= 0:
        c = mezclar(horizonte, cenit, y ** 0.6)
    else:
        # Debajo del horizonte: un mar de nubes, como si la isla flotara
        # muy por encima de ellas.
        c = mezclar((0.80, 0.86, 0.94), (0.58, 0.68, 0.82), (-y) ** 0.6)
        if y < -0.02:
            px, pz = x / -y * 0.35, z / -y * 0.35
            n = fbm(px + 50, pz + 50)
            nube = suave(0.40, 0.62, n) * suave(0.02, 0.18, -y)
            luz = 0.88 + 0.12 * suave(0.45, 0.75, fbm(px * 2.1 + 9, pz * 2.1 + 9))
            c = mezclar(c, (0.97 * luz, 0.97 * luz, 1.0 * luz), nube * 0.95)
    if y > 0.03:
        px, pz = x / y * 0.55, z / y * 0.55
        n = fbm(px + 20, pz + 20)
        nube = suave(0.52, 0.72, n) * suave(0.03, 0.3, y)
        sombra = 0.86 + 0.14 * suave(0.5, 0.8, fbm(px * 1.7 + 5, pz * 1.7 + 5))
        c = mezclar(c, (0.98 * sombra, 0.98 * sombra, 1.0 * sombra), nube * 0.92)
    dot = x * DIR_SOL[0] + y * DIR_SOL[1] + z * DIR_SOL[2]
    if dot > 0:
        c = tuple(c[i] + (1.0, 0.85, 0.55)[i] * (dot ** 10) * 0.18 for i in range(3))
        c = tuple(c[i] + (1.0, 0.95, 0.8)[i] * (dot ** 400) * 0.9 for i in range(3))
    if dot > math.cos(math.radians(2.2)):
        c = (1.0, 0.98, 0.9)
    return tuple(min(1.0, v) * 255 for v in c)


CARAS = {
    "px": lambda s, t: (1, t, s),
    "nx": lambda s, t: (-1, t, -s),
    "py": lambda s, t: (s, 1, -t),
    "ny": lambda s, t: (s, -1, t),
    "pz": lambda s, t: (-s, t, 1),
    "nz": lambda s, t: (s, t, -1),
}


def skybox(tam=256):
    for nombre, f in CARAS.items():
        pix = []
        for j in range(tam):
            for i in range(tam):
                s = 2 * (i + 0.5) / tam - 1
                t = 1 - 2 * (j + 0.5) / tam
                pix.append(color_cielo(normalizar(f(s, t))))
        guardar_bmp(os.path.join(RAIZ, "skybox", nombre + ".bmp"), tam, tam, pix)


if __name__ == "__main__":
    os.makedirs(os.path.join(RAIZ, "textures"), exist_ok=True)
    os.makedirs(os.path.join(RAIZ, "skybox"), exist_ok=True)
    textura("pasto_arriba", pasto_arriba, 1)
    textura("pasto_lado", pasto_lado, 2)
    textura("tierra", tierra, 3)
    textura("piedra", piedra, 4)
    textura("arena", arena, 5)
    textura("tronco_lado", tronco_lado, 6)
    textura("tronco_arriba", tronco_arriba, 7)
    textura("tablones", tablones, 8)
    textura("hojas", hojas, 9)
    textura("agua", agua, 10)
    textura("vidrio", vidrio, 11)
    textura("amatista", amatista, 12)
    skybox()
    print("Texturas y skybox generados en", RAIZ)
