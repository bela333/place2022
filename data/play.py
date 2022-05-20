from numpy import imag
import pandas

import pygame
from pygame.locals import *
from PIL import Image
from PIL import ImageColor

if __name__=="__main__":
    chunksize = 100000
    colors = set()

    image = Image.new("RGB", size=(2000, 2000), color=(255, 255, 255))


    pygame.init()
    screen = pygame.display.set_mode((1000, 1000), HWSURFACE | DOUBLEBUF | RESIZABLE)

    print("Reading file")
    with pandas.read_csv("sorted.csv.gz",
                            chunksize=chunksize,
                            compression='gzip',
                            names=["date", "uid", "color", "pos"]) as reader:
        for df in reader:
            pygame.event.pump()
            for (_, pixel) in df.iterrows():
                _pos = pixel['pos'].split(',')
                if len(_pos) > 2:
                    continue
                [x, y] = _pos
                x = int(x)
                y = int(y)
                color = ImageColor.getrgb(pixel['color'])
                image.putpixel((x, y), color)
            screen.blit(pygame.transform.scale(pygame.image.fromstring(image.tobytes(), image.size, image.mode), screen.get_size()), (0, 0))
            pygame.display.flip()