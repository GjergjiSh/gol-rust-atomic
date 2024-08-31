import pygame
import csv

# Constants
H = 100  # Height of the grid
W = 100  # Width of the grid
PIXEL_SIZE = 10  # Size of each pixel
WINDOW_HEIGHT = H * PIXEL_SIZE
WINDOW_WIDTH = W * PIXEL_SIZE

# Initialize Pygame
pygame.init()
screen = pygame.display.set_mode((WINDOW_WIDTH, WINDOW_HEIGHT))
pygame.display.set_caption("Pixel Drawing")

# Initialize the pixel state grid
pixel_state = [[0 for _ in range(W)] for _ in range(H)]

def save_to_csv(filename):
    with open(filename, mode='w', newline='') as file:
        writer = csv.writer(file)
        writer.writerows(pixel_state)

def main():
    running = True
    drawing = False

    while running:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False
            elif event.type == pygame.MOUSEBUTTONDOWN:
                drawing = True
            elif event.type == pygame.MOUSEBUTTONUP:
                drawing = False

        if drawing:
            mouse_x, mouse_y = pygame.mouse.get_pos()
            grid_x = mouse_x // PIXEL_SIZE
            grid_y = mouse_y // PIXEL_SIZE
            if 0 <= grid_x < W and 0 <= grid_y < H:
                pixel_state[grid_y][grid_x] = 1

        # Draw the pixels
        screen.fill((255, 255, 255))  # Fill the screen with white
        for y in range(H):
            for x in range(W):
                if pixel_state[y][x] == 1:
                    pygame.draw.rect(screen, (0, 0, 0), (x * PIXEL_SIZE, y * PIXEL_SIZE, PIXEL_SIZE, PIXEL_SIZE))

        pygame.display.flip()

    pygame.quit()
    save_to_csv('output/pixel_state.csv')

if __name__ == "__main__":
    main()