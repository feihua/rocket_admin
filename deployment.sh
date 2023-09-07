#!/bin/bash

cargo build -r

#停止服务
docker stop rocket-admin-sea


#删除容器
docker rm -f rocket-admin-sea

#删除镜像
docker rmi -f rocket-admin-sea:v1

#删除none镜像
docker rmi -f $(docker images | grep "none" | awk '{print $3}')

#构建服务
docker build -t rocket-admin-sea:v1 -f Dockerfile .

#启动服务
docker run -itd --net=host --name=rocket-admin-sea rocket-admin-sea:v1
