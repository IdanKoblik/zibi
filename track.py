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
    num_hands=2,
)
detector = vision.HandLandmarker.create_from_options(options)

AIR_ZONE = (0.35, 0.35, 0.95, 0.95)
INDEX_TIP = 8


def point_in_zone(px, py, zone):
    x1, y1, x2, y2 = zone
    return x1 <= px <= x2 and y1 <= py <= y2

cap = cv2.VideoCapture(0)

pTime = 0
cTime = 0

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
    print(f"{h}, {w}")

    # Air zone in pixel coordinates
    zx1, zy1 = int(AIR_ZONE[0] * w), int(AIR_ZONE[1] * h)
    zx2, zy2 = int(AIR_ZONE[2] * w), int(AIR_ZONE[3] * h)
    zone_active = False
    print(f"air zone corners: TL=({zx1}, {zy1}) TR=({zx2}, {zy1}) "
          f"BL=({zx1}, {zy2}) BR=({zx2}, {zy2})")

    if results.hand_landmarks:
        for hand_idx, handLms in enumerate(results.hand_landmarks):
            points = []
            for id, lm in enumerate(handLms):
                cx, cy = int(lm.x * w), int(lm.y * h)
                points.append((cx, cy))
                print(id, cx, cy)
                if id == 0:
                    cv2.circle(img, (cx, cy), 10, (255, 0, 255), cv2.FILLED)

            tx, ty = points[INDEX_TIP]
            if point_in_zone(tx, ty, (zx1, zy1, zx2, zy2)):
                zone_active = True

            for start_idx, end_idx in HAND_CONNECTIONS:
                cv2.line(img, points[start_idx], points[end_idx], (255, 255, 255), 2)
            for cx, cy in points:
                cv2.circle(img, (cx, cy), 4, (255, 0, 255), cv2.FILLED)

            cv2.circle(img, (tx, ty), 9, (0, 255, 255), cv2.FILLED)

            if results.handedness:
                raw = results.handedness[hand_idx][0].category_name
                label = {"Left": "Right", "Right": "Left"}.get(raw, raw)
            else:
                label = f"Hand {hand_idx + 1}"
            text = f"{label}: ({tx}, {ty})"
            cv2.putText(img, text, (w - 320, 30 + hand_idx * 30),
                        cv2.FONT_HERSHEY_PLAIN, 1.5, (0, 255, 0), 2)

    zone_color = (0, 255, 0) if zone_active else (160, 160, 160)
    cv2.rectangle(img, (zx1, zy1), (zx2, zy2), zone_color, 3)
    cv2.putText(img, "ACTIVE" if zone_active else "AIR ZONE",
                (zx1 + 8, zy1 + 28), cv2.FONT_HERSHEY_PLAIN, 1.5, zone_color, 2)
    
    for (px, py), txt in [((zx1, zy1), f"({zx1},{zy1})"),
                          ((zx2, zy1), f"({zx2},{zy1})"),
                          ((zx1, zy2), f"({zx1},{zy2})"),
                          ((zx2, zy2), f"({zx2},{zy2})")]:
        cv2.putText(img, txt, (px + 4, py - 6),
                    cv2.FONT_HERSHEY_PLAIN, 1, zone_color, 1)

    cTime = time.time()
    fps = 1 / (cTime - pTime) if pTime else 0
    pTime = cTime

    cv2.putText(img, str(int(fps)), (10, 70), cv2.FONT_HERSHEY_PLAIN, 3, (255, 0, 255), 3)

    cv2.imshow("Image", img)
    if cv2.waitKey(1) & 0xFF == ord('q'):
        break

cap.release()
cv2.destroyAllWindows()

