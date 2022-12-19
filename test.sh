dfx stop
sudo lsof -nP | grep LISTEN | grep -E "800" | awk '{print $2}' | xargs kill
dfx start --background --clean
dfx deploy verify
#dfx deploy xidc
#dfx deploy xid --argument '(principal "77owi-ydjey-cht3l-nifhw-xkeio-jalgg-bikxb-kl6qa-ccciy-ztlm3-eqe")'