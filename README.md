# rocket-admin

是基于rocket和sea orm的rbac权限管理系统

# 生成entity

sea-orm-cli generate entity -o src/model --with-serde both

# 前端项目

```
https://github.com/feihua/antd-admin.git
```

# 相关rust web框架项目

```
https://github.com/feihua/actix-admin.git # actix-web框架
https://github.com/feihua/axum-admin.git  # axum框架
https://github.com/feihua/salvo-admin.git # salvo-web框架
https://github.com/feihua/ntex-admin.git # ntex-web框架
```

# 预览地址

http://139.159.180.129:81/salvo 账号：18613030352 密码：123456

# 本地启动

```
1.创建数据库并导入sql脚本
2.修改setup.rs代码中的 const DATABASE_URL: &str = "mysql://root:ad879037-c7a4-4063-9236-6bfc35d54b7d@139.159.180.129:3306/rustdb"; 为你自己的数据信息
3.启动 cargo run main.rs
4.验证脚本在docs目录下,请求接口前要先执行登录接口(user.http文件中)

POST {{host}}/api/login
Content-Type: application/json

{
  "mobile": "18613030352",
  "password": "123456"
}
> {% client.global.set("token", response.body.data.token); %}

```

# 系统截图

## 首页

![home](docs/images/home.png)

## 用户界面

![user](docs/images/user.png)

## 角色分配界面

![user-role](docs/images/user-role.png)

## 角色界面

![role](docs/images/role.png)

## 菜单分配界面

![role-menu](docs/images/role-menu.png)

## 菜单界面

![menu](docs/images/menu.png)
