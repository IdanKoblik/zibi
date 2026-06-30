import sys
import cv2
import time
import mediapipe as mp
from mediapipe.tasks import python
from mediapipe.tasks.python import vision

HAND_CONNECTIONS = [
    (0, 1), (1, 2), (2, 3), (3, 4),          # thumb
    (0, 5), (5, 6), (6, 7), (7, 8),          # index finger
    (5, 9), (9, 10), (10, 11), (11, 12),     # middle finger
    (9, 13), (13, 14), (14, 15), (15, 16),   # ring finger
    (13, 17), (17, 18), (18, 19), (19, 20),  # pinky
    (0, 17),                                 # palm base
]

options = vision.HandLandmarkerOptions(
    base_options=python.BaseOptions(model_asset_path="hand_landmarker.task"),
    running_mode=vision.RunningMode.VIDEO,
    num_hands=1,
)
detector = vision.HandLandmarker.create_from_options(options)

TIP_INDEX = 8

cap = cv2.VideoCapture(0)

p_time = 0
c_time = 0

while True:
    success, img = cap.read()
    if not success:
        break

    img = cv2.flip(img, 1)

    imgRGB = cv2.cvtColor(img, cv2.COLOR_BGR2RGB)
    mp_image = mp.Image(image_format=mp.ImageFormat.SRGB, data=imgRGB)

    timestamp_ms = int(time.time() * 1000)
    results = detector.detect_for_video(mp_image, timestamp_ms)

    h, w, c = img.shape
    if not results.hand_landmarks:
        continue

    for hand_idx, handLms in enumerate(results.hand_landmarks):
        if results.handedness:
            raw = results.handedness[hand_idx][0].category_name
            label = {"Left": "Right", "Right": "Left"}.get(raw, raw)
        else:
            label = f"Hand {hand_idx + 1}"

        if label != "Right":
            continue

        points = []
        for id, lm in enumerate(handLms):
            cx, cy = int(lm.x * w), int(lm.y * h)
            points.append((cx, cy))
            print(id, cx, cy, file=sys.stderr)
            if id == 0:
                cv2.circle(img, (cx, cy), 10, (255, 0, 255), cv2.FILLED)

        tx, ty = points[TIP_INDEX]
        for start_idx, end_idx in HAND_CONNECTIONS:
            cv2.line(img, points[start_idx], points[end_idx], (255, 255, 255), 2)
        for cx, cy in points:
            cv2.circle(img, (cx, cy), 4, (255, 0, 255), cv2.FILLED)

        cv2.circle(img, (tx, ty), 9, (0, 255, 255), cv2.FILLED)

        text = f"{label}: ({tx}, {ty})"
        cv2.putText(img, text, (w - 320, 30 + hand_idx * 30), cv2.FONT_HERSHEY_PLAIN, 1.5, (0, 255, 0), 2)

        print(f"{tx, ty}")
        sys.stdout.flush()

    c_time = time.time()
    fps = 1 / (c_time - p_time) if p_time else 0
    p_time = c_time

    cv2.putText(img, f"FPS: {int(fps)}", (10, 70), cv2.FONT_HERSHEY_PLAIN, 3, (255, 0, 255), 3)

    cv2.imshow("Image", img)
    if cv2.waitKey(1) & 0xFF == ord('q'):
        break

cap.release()
cv2.destroyAllWindows()

