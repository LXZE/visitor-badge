docker run -d \
 -v "${PWD}"/data:/data \
 -p 8080:8080 \
 --name visitor-badge \
 visitor_badge \
 sh -c "cd /app; diesel migration run; visitor-badge"