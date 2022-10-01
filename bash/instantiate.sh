source load_env.sh 

CODE_ID=921 

# Create the init msg 
INIT={}

# Init the contract 
wasmd tx wasm instantiate $CODE_ID "$INIT" --from wallet --label "Counter" $TXFLAG -y --no-admin  
sleep 2

# Get the contract addr 
CONTRACT=$(wasmd query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')
echo "Contract instantiated: ${CONTRACT}"
