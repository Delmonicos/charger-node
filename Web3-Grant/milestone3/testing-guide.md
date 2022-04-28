# milestone 3 : Testing Guide

This document contains a guide for testing the application in the scope of milestone 3.

## Scope

The objective of this milestone 3 is to implement 2 webapps to demonstrate the Delmonicos proof-of-concept:
- **Admin Frontend**: supervision application used to monitor chargers status, to view charging sessions history for all users
- **User Frontend**: (responsive) web application allowing a end-user to check the status of nearby chargers on a map, and to start / stop a charge on a specific charging point


The full demonstration of these two web application (and usage of Substrate node) can be seen in the demonstration video here: [https://youtu.be/AlJrFuhhVN4](https://youtu.be/AlJrFuhhVN4)

To start the blockchain node required for this milestone, please follow the instructions here: [milestone2/testing-guide](https://github.com/Delmonicos/charger-node/blob/main/Web3-Grant/milestone2/testing-guide.md#build-and-run-the-chain-in-development-mode).

Both web applications must be run with node v16.

## Admin Frontend

Project URL: [https://github.com/Delmonicos/charging-management-platform](https://github.com/Delmonicos/charging-management-platform)

The objective of this administration interface is to prototype the management UI for a charge points operator.

Through this interface, the network administrator can declare a new charger in the infrastructure. 
Adding a charger requires specifying the GPS coordinates of the charge point. This information is stored as attributes of the DID representing the charge point.  

The administrator also has the possibility to define the price per KwH that will be charged to the users using one of the charge points of the network.   
Here the mechanism is deliberately oversimplified: in a real use case the price grid stored in the blockchain will be more complex (price depending on the time, on the charger, ...).

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

The User Frontend is a responsive web application designed to prototype the experience of the EV-driver on his mobile.

When the user loads the application, it will list the different charging stations available in the neighborhood.   
This information is directly retrieved from the blockchain, based on the attributes associated with the DIDs of the charging points.  

When the user wants to start his first charge session, he has to register in the platform.  
Technically, a wallet is generated on the mobile, and a consent is stored on the blockchain (this information represents a payment authorization that will be used at the end of the charge sessions).  
When the user starts the charge, he signs a transaction with his wallet stored on the mobile.  
Then the charge session starts and the interraction with the charging point is executed as defined in the pallet "charge_session".

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
