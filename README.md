# Flight Tracking ERAU SE300 
[![Crates.io](https://img.shields.io/crates/v/flight_tracking_erau_se300.svg)](https://crates.io/crates/flight_tracking_erau_se300) 
[![build](https://shields.io/github/workflow/status/FlightTrackingERAU/FlightTracking/CI/develop)](https://github.com/FlightTrackingERAU/FlightTracking/actions)
[![docs.rs](https://img.shields.io/docsrs/flight_tracking_erau_se300)](https://docs.rs/flight_tracking_erau_se300/0.3.0/flight_tracking_erau_se300/)
 [![License](https://img.shields.io/crates/l/flight_tracking_erau_se300)](https://github.com/FlightTrackingERAU/FlightTracking/blob/master/LICENSE)
[![Dependency](https://deps.rs/repo/github/FlightTrackingERAU/FlightTracking/status.svg)](https://deps.rs/repo/github/FlightTrackingERAU/FlightTracking)

## Description

Software that allows for weather and plane tracking to facilitate the user in looking at plane paths. Many people who choose flights are forced to change flights or wait, when then get canceled or delayed due to weather. For some people it is fine but those who have deadlines would want to avoid this. Buy allowing flights and weather to be tracked it is possible for the user to avoid these delays and flight cancelations.

This is a class project for **Embry–Riddle Aeronautical University**, class **SE 300** (Software Engineer Practices).

## Language
Rust: https://www.rust-lang.org/

## Implementations
* Zoom: 

![Zooming Gif](https://github.com/FlightTrackingERAU/FlightTracking/blob/master/examples/gif/ezgif.com-gif-maker.gif)


* Filter Planes by Airline

![Filter Gif](https://github.com/FlightTrackingERAU/FlightTracking/blob/master/examples/gif/airline-filter.gif)


* Toggle Weather on/off

![Weather Gif](https://github.com/FlightTrackingERAU/FlightTracking/blob/master/examples/gif/weather-toggle.gif)

* Toggle Airports on/off:

![Airport On/Off Gif](https://github.com/FlightTrackingERAU/FlightTracking/blob/master/examples/gif/airport_toggle.gif)

# Guide

## Navigation

The Flight Tracking app allows the user to move freely in the world. The user may zoom in or zoom out as much as they want as long as is in the valid ranges.
 
##### Zoom

* **Scroll Up**: Zooms Out
* **Scroll Down**: Zooms In

##### Movement

The user must **Hold-Left-Click** in order to be able to move around the map. While Holding, user can just move the mouse to their preferrable location.

## UI

There are a total of 11 buttons on the UI. 6 of this buttons are for filtering purposes like, filtering planes according to their airlines. The other 5 buttons are display settings such as showing weather or showing airports. 

#### Buttons

* **Airplane Button**: ![Airplane Button](/examples/pictures/airplane-button.png)

This button displays all the filtering options for planes. 
When clicked 6 filter-type buttons will appear next to the **Airplane Button** 

* **Weather Button**: ![Weather Button](/examples/pictures/weather-button.png)

This button enables/disables the weather on map. (Default = Disabled)

* **Debug Button**: ![Debug Button](/examples/pictures/debug-button.png) 

This button just displays debug info to the user on the top left of the screen. **Debug** includes FPS, Speed of Map Rendering, Speed of Weather Rendering, and more features. 

* **Airport Button**: ![Airport Button](/examples/pictures/airport-button.png)

This button displays the airport. Clicking it will enable/disable the airports on the screen. (Default = Enabled)

* **Bench Button**: ![Bench Button](/examples/pictures/strong-button.png)

This button outputs into the **console** the speed of events the user do on the UI. Mostly for developers to use. 


* **Filter Buttons**: 

![Filter Buttons](/examples/pictures/filter-button.png)

This are the **Plane Filter** Buttons. When any one type of Filter Button is clicked. The Planes in the map will change according to the Filter(or Airline). Example, if user clicked **American Airlines** only planes form American Airlines will display on the map.  
 

# To-Do:

* When user clicks a plane, program should display some basic data of the plane in the screen according to the plane's position.


# Known-Issues:

* The Map doesn't load after using it for while. This is because the Map API used in the programs limits the amount of tiles it can load. 

* (Not in Issue, more like a prefer) The Plane API only gets data every 5 seconds or so. Preferrably another Plane API with faster data should be found.
