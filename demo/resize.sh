ffmpeg -i amu-demo.gif \
  -vf "fps=10,scale=900:-1:flags=lanczos" \
  amu-demo.min.gif
