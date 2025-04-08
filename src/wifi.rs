use esp_idf_svc::eventloop::{EspEventLoop, System};
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi};

pub fn connect(
    esp_wifi: &mut EspWifi<'_>,
    sysloop: EspEventLoop<System>,
    ssid: impl AsRef<str>,
    password: impl AsRef<str>,
) {
    let mut wifi = BlockingWifi::wrap(esp_wifi, sysloop).unwrap();

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid.as_ref().try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        password: password.as_ref().try_into().unwrap(),
        ..Default::default()
    }))
    .unwrap();

    wifi.start().unwrap();
    wifi.connect().unwrap();
    wifi.wait_netif_up().unwrap();
}
