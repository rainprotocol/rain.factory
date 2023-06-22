// SPDX-License-Identifier: CAL
pragma solidity =0.8.18;

import "./interface/ICloneableV1.sol";
import "./interface/ICloneFactoryV1.sol";
import "rain.interpreter/abstract/DeployerDiscoverableMetaV1.sol";
import {Clones} from "openzeppelin-contracts/contracts/proxy/Clones.sol";

/// Thrown when an implementation is the zero address which is always a mistake.
error ZeroImplementation();

bytes32 constant CLONE_FACTORY_META_HASH = bytes32(
    0xae0fb5b68fe1791c72509bf46ea6abf6a982d21451265be0a017f7959712a67e
);

contract CloneFactory is ICloneableFactoryV1, DeployerDiscoverableMetaV1 {
    constructor(
        DeployerDiscoverableMetaV1ConstructionConfig memory config_
    ) DeployerDiscoverableMetaV1(CLONE_FACTORY_META_HASH, config_) {}

    /// @inheritdoc ICloneableFactoryV1
    function clone(
        address implementation_,
        bytes calldata data_
    ) external returns (address) {
        if (implementation_ == address(0)) {
            revert ZeroImplementation();
        }
        address clone_ = Clones.clone(implementation_);
        emit NewClone(msg.sender, implementation_, clone_);
        ICloneableV1(clone_).initialize(data_);
        return clone_;
    }
}
