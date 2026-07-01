import sys
import cv2
import time
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import mediapipe as mp
from mediapipe.tasks import python
from mediapipe.tasks.python import vision


class MJPEGServer:
    def __init__(self, host="127.0.0.1", port=8080):
        self.host = host
        self.port = port
        self._frame = None
        self._lock = threading.Lock()
        server = self

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *args):
                pass  # keep stdout/stderr clean for the zibi pipe

            def do_GET(self):
                if self.path in ("/", "/index.html"):
                    self.send_response(200)
                    self.send_header("Content-Type", "text/html")
                    self.end_headers()
                    self.wfile.write(
                        b"<html><body style='margin:0;background:#111'>"
                        b"<img src='/stream' style='width:100vw;height:100vh;object-fit:contain'>"
                        b"</body></html>"
                    )
                    return
                if self.path != "/stream":
                    self.send_error(404)
                    return

                self.send_response(200)
                self.send_header(
                    "Content-Type", "multipart/x-mixed-replace; boundary=frame"
                )
                self.end_headers()
                try:
                    while True:
                        with server._lock:
                            frame = server._frame
                        if frame is None:
                            time.sleep(0.01)
                            continue
                        self.wfile.write(b"--frame\r\n")
                        self.wfile.write(b"Content-Type: image/jpeg\r\n\r\n")
                        self.wfile.write(frame)
                        self.wfile.write(b"\r\n")
                        time.sleep(1 / 60)
                except (BrokenPipeError, ConnectionResetError):
                    pass  # browser tab closed

        self._httpd = ThreadingHTTPServer((host, port), Handler)

    def start(self):
        thread = threading.Thread(target=self._httpd.serve_forever, daemon=True)
        thread.start()
        print(f"MJPEG stream: http://{self.host}:{self.port}", file=sys.stderr)
        return self

    def update(self, img):
        ok, jpg = cv2.imencode(".jpg", img)
        if ok:
            with self._lock:
                self._frame = jpg.tobytes()

class VideoStream:
    def __init__(self, src=0):
        self.cap = cv2.VideoCapture(src)
        self.cap.set(cv2.CAP_PROP_FPS, 60)
        self.cap.set(cv2.CAP_PROP_BUFFERSIZE, 1)
        self.grabbed, self.frame = self.cap.read()
        self.started = False
        self.read_lock = threading.Lock()

    def start(self):
        if self.started:
            return self
        self.started = True
        self.thread = threading.Thread(target=self.update, args=())
        self.thread.daemon = True
        self.thread.start()
        return self

    def update(self):
        while self.started:
            grabbed, frame = self.cap.read()
            if not grabbed:
                self.started = False
                break
            with self.read_lock:
                self.grabbed = grabbed
                self.frame = frame

    def read(self):
        with self.read_lock:
            if not self.grabbed or self.frame is None:
                return False, None
            return self.grabbed, self.frame.copy()

    def release(self):
        self.started = False
        self.thread.join(timeout=1.0)
        self.cap.release()

HAND_CONNECTIONS = [
    (0, 1), (1, 2), (2, 3), (3, 4),          # thumb
    (0, 5), (5, 6), (6, 7), (7, 8),          # index finger
    (5, 9), (9, 10), (10, 11), (11, 12),     # middle finger
    (9, 13), (13, 14), (14, 15), (15, 16),   # ring finger
    (13, 17), (17, 18), (18, 19), (19, 20),  # pinky
    (0, 17),                                 # palm base
]

options = vision.HandLandmarkerOptions(
    base_options=python.BaseOptions(model_asset_path="models/hand_landmarker.task"),
    running_mode=vision.RunningMode.VIDEO,
    num_hands=1,
)
detector = vision.HandLandmarker.create_from_options(options)

TIP_INDEX = 8

vs = VideoStream(0).start()
stream = MJPEGServer().start()

p_time = 0
c_time = 0

while True:
    success, img = vs.read()
    if not success or img is None:
        time.sleep(0.001)
        continue

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
            # print(id, cx, cy, file=sys.stderr)
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

        print(f"{tx}, {ty}")
        sys.stdout.flush()

    c_time = time.time()
    fps = 1 / (c_time - p_time) if p_time else 0
    p_time = c_time

    cv2.putText(img, f"FPS: {int(fps)}", (10, 70), cv2.FONT_HERSHEY_PLAIN, 3, (255, 0, 255), 3)

    stream.update(img)

vs.release()

