# milestone 3 : Testing Guide

This document contains a guide for testing the application in the scope of milestone 3.

## Scope

The objective of this milestone 3 is to implement 2 webapps to demonstrate the Delmonicos proof-of-concept:
- **Admin Frontend**: supervision application used to monitor chargers status, to view charging sessions history for all users
- **User Frontend**: (responsive) web application allowing a end-user to check the status of nearby chargers on a map, and to start / stop a charge on a specific charging point


The full demonstration of these two web application (and usage of Substrate node) can be seen in the demonstration video here: [https://youtu.be/AlJrFuhhVN4](https://youtu.be/AlJrFuhhVN4)

## Admin Frontend

Project URL: [https://github.com/Delmonicos/charging-management-platform](https://github.com/Delmonicos/charging-management-platform)

### Clone project

```
git clone git@github.com:Delmonicos/charging-management-platform.git
```

### Install dependencies

```
cd charging-management-platform
yarn install
```

### Start the webapp

```
yarn run start
```

Web application is available at: [http://localhost:3000](http://localhost:3000)


## User Frontend

Project URL: [https://github.com/Delmonicos/user-frontend](https://github.com/Delmonicos/user-frontend)

### Clone project

```
git clone git@github.com:Delmonicos/user-frontend.git
```

### Install dependencies

```
cd user-frontend
yarn install
```

### Start the webapp

```
PORT=3001 yarn run start
```

Web application is available at: [http://localhost:3001](http://localhost:3001)