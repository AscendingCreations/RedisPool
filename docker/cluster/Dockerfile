FROM redis:latest
EXPOSE 6379 6380 6381
COPY redis.conf.template .
COPY run.sh .
CMD ["./run.sh"]