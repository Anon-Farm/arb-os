pragma solidity >=0.4.21 <0.7.0;

interface ArbOwner {
    // Support actions that can be taken by the chain's owner.
    // All methods will revert, unless the caller is the chain's owner.

    function giveOwnership(address newOwnerAddr) external;

    // To upgrade ArbOS, the ower calls startArbosUpgrade, then calls continueArbosUpgrade one or more times to upload 
    // the code to be installed as the upgrade, then calls finishArbosUpgrade to complete the upgrade and start executing the new code.
    function startCodeUpload() external;
    function continueCodeUpload(bytes calldata marshalledCode) external;
    function finishCodeUploadAsArbosUpgrade() external;
}

